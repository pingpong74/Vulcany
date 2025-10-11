use smallvec::smallvec;
use std::time::Instant;
use vulcany::*;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

use std::sync::Arc;

vertex!(MyVertex {
    input_rate: VERTEX,
    pos: [f32; 2] => { location: 0, format: R32G32_SFLOAT },
    color: [f32; 3] => { location: 1, format: R32G32B32_SFLOAT },
});

struct FrameData {
    cmd_buffer: CommandBuffer,
    fence: Fence,
    image_semaphore: Semaphore,
    render_finish_semaphore: Semaphore,
}

struct VulkanApp {
    window: Arc<Window>,
    instance: Instance,
    device: Device,
    swapchain: Swapchain,
    pipeline_manager: PipelineManager,
    raster_pipeline: RasterizationPipeline,
    vertex_buffer: BufferID,
    frame_data: FrameData,
}

impl VulkanApp {
    fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let window_attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescription {
            api_version: ApiVersion::VkApi1_3,
            enable_validation_layers: true,
            window: window.clone(),
        });

        let device = instance.create_device(&DeviceDescription {
            use_compute_queue: true,
            use_transfer_queue: true,
        });

        let swapchain = device.create_swapchain(&SwapchainDescription {
            image_count: 3,
            width: size.width,
            height: size.height,
        });

        let pipeline_manager = device.create_pipeline_manager("examples/shaders");
        let raster_pipeline =
            pipeline_manager.create_rasterization_pipeline(&RasterizationPipelineDescription {
                vertex_input: MyVertex::vertex_input_description(),
                vertex_shader_path: "vertex_shader.slang",
                fragment_shader_path: "fragment_shader.slang",
                cull_mode: CullMode::None,
                front_face: FrontFace::Clockwise,
                polygon_mode: PolygonMode::Fill,
                depth_stencil: DepthStencilOptions::default(),
                alpha_blend_enable: false,
                outputs: PipelineOutputs {
                    color: vec![Format::Rgba16Float], // color attaachment in dynmic rendering
                    depth: None,
                    stencil: None,
                },
            });

        let vertex_data = [
            MyVertex {
                pos: [0.5, 0.5],
                color: [0.2, 0.2, 0.8],
            },
            MyVertex {
                pos: [-0.5, 0.5],
                color: [0.2, 0.8, 0.2],
            },
            MyVertex {
                pos: [0.0, -0.5],
                color: [0.8, 0.2, 0.2],
            },
        ];

        let staging_buffer = device.create_buffer(&BufferDescription {
            usage: BufferUsage::TRANSFER_SRC,
            size: 60,
            memory_type: MemoryType::PreferHost,
            create_mapped: true,
        });

        device.write_data_to_buffer(staging_buffer, &vertex_data);

        let vertex_buffer = device.create_buffer(&BufferDescription {
            usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX,
            size: 60,
            memory_type: MemoryType::DeviceLocal,
            create_mapped: false,
        });

        let cmd = device.allocate_command_buffer(CommandBufferLevel::Primary, QueueType::Transfer);
        cmd.begin_recording(CommandBufferUsage::OneTimeSubmit);
        cmd.copy_buffer(&BufferCopyInfo {
            src_buffer: staging_buffer,
            dst_buffer: vertex_buffer,
            size: 60,
            src_offset: 0,
            dst_offset: 0,
        });
        cmd.end_recording();
        device.submit(&QueueSubmitInfo {
            fence: None,
            command_buffers: smallvec![cmd.clone()],
            wait_semaphores: smallvec![],
            signal_semaphores: smallvec![],
        });
        device.wait_queue(QueueType::Transfer);
        device.destroy_buffer(staging_buffer);
        device.free_command_buffer(cmd);

        return VulkanApp {
            frame_data: FrameData {
                cmd_buffer: device
                    .allocate_command_buffer(CommandBufferLevel::Primary, QueueType::Graphics),
                fence: device.create_fence(true),
                image_semaphore: device.create_binary_semaphore(),
                render_finish_semaphore: device.create_binary_semaphore(),
            },
            window: window,
            instance: instance,
            device: device,
            swapchain: swapchain,
            pipeline_manager: pipeline_manager,
            raster_pipeline: raster_pipeline,
            vertex_buffer: vertex_buffer,
        };
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.device.wait_idle();
        let new_swapchain = self.device.recreate_swapchain(
            &SwapchainDescription {
                image_count: 3,
                width: width,
                height: height,
            },
            &self.swapchain,
        );
        let old_swapchain = std::mem::replace(&mut self.swapchain, new_swapchain);
        drop(old_swapchain);
    }

    fn render(&self) {
        let start = Instant::now();
        let size = self.window.inner_size();

        if size.width == 0 || size.height == 0 {
            return;
        }

        //self.device.wait_idle();
        self.device.wait_fence(self.frame_data.fence);
        self.device.reset_fence(self.frame_data.fence);

        let (img, img_view) = self
            .swapchain
            .acquire_image(Some(&self.frame_data.image_semaphore), None);

        self.device.reset_command_pool(QueueType::Graphics);

        self.frame_data
            .cmd_buffer
            .begin_recording(CommandBufferUsage::OneTimeSubmit);

        self.frame_data
            .cmd_buffer
            .pipeline_barrier(&[Barrier::Image {
                image: img,
                old_layout: ImageLayout::Undefined,
                new_layout: ImageLayout::ColorAttachment,
                src_stage: PipelineStage::TopOfPipe,
                dst_stage: PipelineStage::ColorAttachmentOutput,
                src_access: AccessType::None,
                dst_access: AccessType::ColorAttachmentWrite,
                base_mip: 0,
                level_count: 1,
                base_layer: 0,
                layer_count: 1,
            }]);

        self.frame_data
            .cmd_buffer
            .begin_rendering(&RenderingBeginInfo {
                render_area: RenderArea {
                    offset: 0,
                    width: size.width,
                    height: size.height,
                },
                rendering_flags: RenderingFlags::None,
                view_mask: 0,
                layer_count: 1,
                color_attachments: vec![RenderingAttachment {
                    image_view: img_view,
                    image_layout: ImageLayout::ColorAttachment,
                    clear_value: ClearValue::ColorFloat([0.2, 0.2, 0.4, 1.0]),
                    ..Default::default()
                }],
                depth_attachment: None,
                stencil_attachment: None,
            });

        self.frame_data
            .cmd_buffer
            .bind_rasterization_pipeline(&self.raster_pipeline);
        self.frame_data
            .cmd_buffer
            .set_viewport_and_scissor(size.width, size.height);
        self.frame_data
            .cmd_buffer
            .bind_vertex_buffer(self.vertex_buffer, 0);
        self.frame_data.cmd_buffer.draw(3, 1, 0, 0);

        self.frame_data.cmd_buffer.end_rendering();
        self.frame_data
            .cmd_buffer
            .pipeline_barrier(&[Barrier::Image {
                image: img,
                old_layout: ImageLayout::ColorAttachment,
                new_layout: ImageLayout::PresentSrc,
                src_stage: PipelineStage::ColorAttachmentOutput,
                dst_stage: PipelineStage::BottomOfPipe,
                src_access: AccessType::ColorAttachmentWrite,
                dst_access: AccessType::None,
                base_mip: 0,
                level_count: 1,
                base_layer: 0,
                layer_count: 1,
            }]);
        self.frame_data.cmd_buffer.end_recording();

        self.device.submit(&QueueSubmitInfo {
            fence: Some(self.frame_data.fence),
            command_buffers: smallvec![self.frame_data.cmd_buffer.clone()],
            wait_semaphores: smallvec![SemaphoreInfo {
                semaphore: self.frame_data.image_semaphore,
                pipeline_stage: PipelineStage::ColorAttachmentOutput,
                value: None
            }],
            signal_semaphores: smallvec![SemaphoreInfo {
                semaphore: self.frame_data.render_finish_semaphore,
                pipeline_stage: PipelineStage::BottomOfPipe,
                value: None
            }],
        });

        self.swapchain
            .present(&[self.frame_data.render_finish_semaphore]);

        let duration = start.elapsed();
        //panic!()
        //println!("{}", duration.as_micros());
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_buffer(self.vertex_buffer);
        self.device.destroy_fence(self.frame_data.fence);
        self.device
            .destroy_semaphore(self.frame_data.image_semaphore);
        self.device
            .destroy_semaphore(self.frame_data.render_finish_semaphore);
        self.device
            .free_command_buffer(self.frame_data.cmd_buffer.clone());
    }
}

#[allow(unused)]
impl ApplicationHandler for VulkanApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                self.render();
                self.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop: EventLoop<()> = EventLoop::with_user_event()
        .build()
        .expect("Failed to create event loop");

    let mut app = VulkanApp::new(&event_loop);

    event_loop.run_app(&mut app).expect("SMT SMT");
}
