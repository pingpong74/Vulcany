use crate::{BufferID, CommandBuffer, ImageID};

pub enum PassType {
    Graphic,
    Compute,
    Transfer,
}

pub struct ReadResources {
    images: Vec<ImageID>,
    buffers: Vec<BufferID>,
}

pub struct WriteResources {
    images: Vec<ImageID>,
    buffers: Vec<BufferID>,
}

pub struct Pass {
    name: String,
    pass_type: PassType,
    read_resources: ReadResources,
    write_resources: WriteResources,
    record: fn(&mut CommandBuffer, ReadResources, WriteResources),
}

struct Edge {}

pub struct TaskGraph {
    passes: Vec<Pass>,
    edges: Vec<Edge>,
}

impl TaskGraph {
    pub fn accquire_image() {}

    pub fn add_pass(pass: Pass) {}

    pub fn present_accquired_image() {}
}
