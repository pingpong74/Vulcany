use vulcany::*;
use winit::{event_loop::EventLoop, window::Window};

use std::sync::Arc;

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

    let buffer = device.create_buffer(&BufferDescription {
        usage: BufferUsage::STAGING,
        size: 10,
        memory_type: MemoryType::AUTO,
    });

    let image = device.create_image(&ImageDescription {
        width: 200,
        height: 200,
        depth: 1,
        ..Default::default()
    });

    let image_view = image.create_image_view(&ImageViewDescription {
        view_type: ImageViewType::TYPE_2D,
        aspect: ImageAspect::DEPTH,
        ..Default::default()
    });

    let sampler = device.create_sampler(&SamplerDescription {
        min_filter: Filter::Nearest,
        ..Default::default()
    });
}
