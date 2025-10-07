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
        enable_validation_layers: true,
        window: window.clone(),
    });

    let device = instance.create_device(&DeviceDescription {
        use_compute_queue: true,
        use_transfer_queue: true,
    });

    let src_buffer_id = device.create_buffer(&BufferDescription {
        usage: BufferUsage::TransferSrc,
        size: 1000,
        memory_type: MemoryType::PreferHost,
        create_mapped: true,
    });

    let dst_buffer_id = device.create_buffer(&BufferDescription {
        usage: BufferUsage::TransferDst,
        size: 1000,
        memory_type: MemoryType::PreferHost,
        create_mapped: false,
    });

    let data = [1, 2, 3, 4, 5];

    device.write_data_to_buffer(src_buffer_id, &data);

    let cmd_buffer =
        device.allocate_command_buffer(CommandBufferLevel::Primary, QueueType::Transfer);

    cmd_buffer.begin_recording(CommandBufferUsage::OneTimeSubmit);
    cmd_buffer.copy_buffer(&BufferCopyInfo {
        src_buffer: src_buffer_id,
        dst_buffer: dst_buffer_id,
        src_offset: 0,
        dst_offset: 0,
        size: (data.len() * 32) as u64,
    });
    cmd_buffer.end_recording();

    device.submit(&QueueSubmitInfo {
        command_buffer_type: QueueType::Transfer,
        fence: None,
        command_buffers: smallvec![cmd_buffer],
        wait_semaphores: smallvec![],
        signal_semaphores: smallvec![],
    });

    device.wait_queue(QueueType::Transfer);

    device.destroy_buffer(src_buffer_id);
    device.destroy_buffer(dst_buffer_id);
}
