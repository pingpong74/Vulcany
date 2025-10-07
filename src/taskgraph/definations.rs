use ash::vk;

#[derive(Clone, Copy)]
pub struct RenderArea {
    offset: u32,
    width: u32,
    height: u32,
}

pub struct RenderingBeginInfo {
    render_area: RenderArea,
}
