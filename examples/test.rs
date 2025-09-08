use vulcany::*;
use winit::{event_loop::EventLoop, window::Window};

use std::sync::Arc;

vertex!(MyVertex {
    input_rate: VERTEX,
    pos: [f32; 2] => { location: 0, format: R32G32_SFLOAT },
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
        api_version: ApiVersion::VK_API_1_2,
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

    let pipeline_manager = device.create_pipeline_manager();

    let buffer = device.create_buffer(&BufferDescription {
        usage: BufferUsage::VERTEX,
        size: 1000,
        memory_type: MemoryType::DEVICE_LOCAL,
    });
}
