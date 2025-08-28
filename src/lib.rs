pub(crate) mod backend;
pub mod core;

#[cfg(test)]
mod tests {

    use crate::core::context::*;
    use winit::{event_loop::EventLoop, window::Window};

    use std::sync::Arc;

    #[cfg(test)]
    fn test() {
        let event_loop: EventLoop<()> = EventLoop::with_user_event()
            .build()
            .expect("Failed to create event loop");

        let window_attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        let context = Context::new(
            &InstanceDescription {
                api_version: ApiVersion::VK_API_1_0,
                enable_validation_layers: true,
                window: window.clone(),
            },
            &DeviceDescription {},
            None,
        );
    }
}
