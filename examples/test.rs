use vulcany::taskgraph::task_graph::*;
use vulcany::*;
use winit::{event_loop::EventLoop, window::Window};

use std::sync::Arc;

vertex!(MyVertex {
    input_rate: VERTEX,
    pos: [f32; 3] => { location: 0, format: R32G32_SFLOAT },
    color: [f32; 3] => { location: 1, format: R32G32B32_SFLOAT },
});

fn test_task(_cmd: &mut CommandBuffer, _read: ReadResources, _write: WriteResources) {}

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
        api_version: ApiVersion::VK_API_1_2,
        enable_validation_layers: true,
        window: window.clone(),
    });

    let device = instance.create_device(&DeviceDescription {
        use_compute_queue: true,
        use_transfer_queue: true,
    });

    let buffer_id = device.create_buffer(&BufferDescription {
        usage: BufferUsage::VERTEX,
        size: 1000,
        ..Default::default()
    });

    device.destroy_buffer(buffer_id);

    let image_id = device.create_image(&ImageDescription::default());

    let image_view_id = device.create_image_view(image_id, &ImageViewDescription::default());

    device.destroy_image_view(image_view_id);
    device.destroy_image(image_id);

    let swapchain = device.create_swapchain(&SwapchainDescription {
        image_count: 3,
        width: size.width,
        height: size.height,
    });

    let pipeline_manager = device.create_pipeline_manager("examples/shaders");

    pipeline_manager.create_rasterization_pipeline(&RasterizationPipelineDescription {
        vertex_input: MyVertex::vertex_input_description(),
        vertex_shader_path: "vertex_shader.slang",
        fragment_shader_path: "fragment_shader.slang",
        cull_mode: CullMode::Back,
        front_face: FrontFace::CounterClockwise,
        polygon_mode: PolygonMode::Fill,
        line_width: 0.5,
        depth_stencil: DepthStencilOptions::default(),
        alpha_blend_enable: true,
        outputs: PipelineOutputs {
            color: vec![ImageFormat::B8G8R8A8_UNORM],
            depth: None,
            stencil: None,
        },
    });

    let mut task_graph = TaskGraph::new(device.clone(), swapchain.clone());

    task_graph.add_pass(Pass {
        name: "Test 0",
        pass_type: PassType::Graphic,
        read_resources: ReadResources {
            images: vec![image_id],
            buffers: vec![buffer_id],
        },
        write_resources: WriteResources {
            images: vec![],
            buffers: vec![buffer_id],
        },
        record: test_task,
    });

    task_graph.add_pass(Pass {
        name: "Test 1",
        pass_type: PassType::Graphic,
        read_resources: ReadResources {
            images: vec![],
            buffers: vec![buffer_id],
        },
        write_resources: WriteResources {
            images: vec![],
            buffers: vec![buffer_id],
        },
        record: test_task,
    });

    task_graph.add_pass(Pass {
        name: "Test 2",
        pass_type: PassType::Graphic,
        read_resources: ReadResources {
            images: vec![image_id],
            buffers: vec![],
        },
        write_resources: WriteResources {
            images: vec![image_id],
            buffers: vec![],
        },
        record: test_task,
    });

    task_graph.add_pass(Pass {
        name: "Test 3",
        pass_type: PassType::Graphic,
        read_resources: ReadResources {
            images: vec![image_id],
            buffers: vec![buffer_id],
        },
        write_resources: WriteResources {
            images: vec![image_id],
            buffers: vec![buffer_id],
        },
        record: test_task,
    });

    task_graph.compile();
}
