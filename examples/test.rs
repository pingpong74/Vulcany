use image::codecs::hdr::SIGNATURE;
use smallvec::smallvec;
use vulcany::*;
use winit::{event_loop::EventLoop, window::Window};

use std::sync::Arc;

vertex!(MyVertex {
    input_rate: VERTEX,
    pos: [f32; 3] => { location: 0, format: R32G32_SFLOAT },
    color: [f32; 3] => { location: 1, format: R32G32B32_SFLOAT },
});

fn main() {
    let event_loop: EventLoop<()> = EventLoop::with_user_event()
        .build()
        .expect("Failed to create event loop");

    let window_attributes = Window::default_attributes();

    let window = Arc::new(
        event_loop
            .create_window(window_attributes)
            .expect("Failed to create window"),
    );

    let size = window.inner_size();

    let instance = Instance::new(&InstanceDescription {
        api_version: ApiVersion::VkApi1_3,
        enable_validation_layers: false,
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

    let fence = device.create_fence(true);
    let image_semaphore = device.create_binary_semaphore();
    let render_finish_semaphore = device.create_binary_semaphore();

    let cmd_buffer =
        device.allocate_command_buffer(CommandBufferLevel::Primary, QueueType::Graphics);

    device.reset_fence(fence);

    let (_, img_view) = swapchain.acquire_image(Some(&image_semaphore), Some(&fence));

    let color_attachment_info = RenderingAttachment {
        image_view: img_view,
        ..Default::default()
    };

    cmd_buffer.begin_recording(CommandBufferUsage::OneTimeSubmit);
    cmd_buffer.begin_rendering(&RenderingBeginInfo {
        render_area: RenderArea {
            offset: 0,
            width: size.width,
            height: size.height,
        },
        rendering_flags: RenderingFlags::None,
        view_mask: 0,
        layer_count: 1,
        color_attachments: vec![color_attachment_info],
        depth_attachment: None,
        stencil_attachment: None,
    });
    cmd_buffer.end_rendering();
    cmd_buffer.end_recording();

    device.submit(&QueueSubmitInfo {
        command_buffer_type: QueueType::Graphics,
        fence: None,
        command_buffers: smallvec![cmd_buffer],
        wait_semaphores: smallvec![SemaphoreInfo {
            semaphore: image_semaphore,
            pipeline_stage: PipelineStage::AllCommands,
            value: None
        }],
        signal_semaphores: smallvec![SemaphoreInfo {
            semaphore: render_finish_semaphore,
            pipeline_stage: PipelineStage::AllCommands,
            value: None
        }],
    });

    swapchain.preset(&render_finish_semaphore);

    device.wait_idle();
}
