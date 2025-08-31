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

    let context = Context::new(
        &InstanceDescription {
            api_version: ApiVersion::VK_API_1_2,
            enable_validation_layers: true,
            window: window.clone(),
        },
        &DeviceDescription {
            use_compute_queue: true,
            use_transfer_queue: true,
        },
        &SwapchainDescription {
            image_count: 3,
            width: size.width,
            height: size.height,
        },
    );
}
