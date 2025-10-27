use crate::*;
use crate::{BufferID, ImageViewID, SamplerID};
use ash::vk;
use std::{ops::BitOr, u64};

////Descriptors////

pub struct BufferWriteInfo {
    pub buffer: BufferID,
    pub offset: u64,
    pub range: u64,
    pub index: u32,
}

impl Default for BufferWriteInfo {
    fn default() -> Self {
        return BufferWriteInfo {
            buffer: BufferID::null(),
            offset: 0,
            range: 0,
            index: 0,
        };
    }
}

pub enum ImageDescriptorType {
    SampledImage,
    StorageImage,
}

pub struct ImageWriteInfo {
    pub view: ImageViewID,
    pub image_descriptor_type: ImageDescriptorType,
    pub index: u32,
}

impl Default for ImageWriteInfo {
    fn default() -> Self {
        return ImageWriteInfo {
            view: ImageViewID::null(),
            image_descriptor_type: ImageDescriptorType::SampledImage,
            index: 0,
        };
    }
}

pub struct SamplerWriteInfo {
    pub sampler: SamplerID,
    pub index: u32,
}

impl Default for SamplerWriteInfo {
    fn default() -> Self {
        return SamplerWriteInfo { sampler: SamplerID::null(), index: 0 };
    }
}

//// Vertex ////

pub trait VertexFormat {
    const FORMAT: Format;
}

impl VertexFormat for f32 {
    const FORMAT: Format = Format::R32Float;
}
impl VertexFormat for [f32; 2] {
    const FORMAT: Format = Format::Rg32Float;
}
impl VertexFormat for [f32; 3] {
    const FORMAT: Format = Format::Rgb32Float;
}
impl VertexFormat for [f32; 4] {
    const FORMAT: Format = Format::Rgba32Float;
}
impl VertexFormat for [u8; 4] {
    const FORMAT: Format = Format::Rgba8Unorm;
}

#[derive(Clone, Copy, Debug)]
pub enum VertexInputRate {
    Vertex,
    Instance,
}

impl VertexInputRate {
    pub fn to_vk(&self) -> ash::vk::VertexInputRate {
        match self {
            VertexInputRate::Vertex => ash::vk::VertexInputRate::VERTEX,
            VertexInputRate::Instance => ash::vk::VertexInputRate::INSTANCE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VertexBinding {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

#[derive(Clone, Copy)]
pub struct VertexAttribute {
    pub location: u32,
    pub binding: u32,
    pub format: Format,
    pub offset: u32,
}

#[derive(Clone)]
pub struct VertexInputDescription {
    pub bindings: Vec<VertexBinding>,
    pub attributes: Vec<VertexAttribute>,
}

impl Default for VertexInputDescription {
    fn default() -> Self {
        return Self {
            bindings: Vec::new(),
            attributes: Vec::new(),
        };
    }
}

impl VertexInputDescription {
    pub fn to_vk(&self) -> (Vec<ash::vk::VertexInputBindingDescription>, Vec<ash::vk::VertexInputAttributeDescription>) {
        let bindings = self
            .bindings
            .iter()
            .map(|b| ash::vk::VertexInputBindingDescription {
                binding: b.binding,
                stride: b.stride,
                input_rate: b.input_rate.to_vk(),
            })
            .collect();

        let attributes = self
            .attributes
            .iter()
            .map(|a| ash::vk::VertexInputAttributeDescription {
                location: a.location,
                binding: a.binding,
                format: a.format.to_vk_format(),
                offset: a.offset,
            })
            .collect();

        (bindings, attributes)
    }
}

//// Rasterization pipeline create info ////
#[derive(Clone, Copy)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

impl CullMode {
    pub(crate) const fn to_vk_flag(&self) -> vk::CullModeFlags {
        match self {
            Self::None => vk::CullModeFlags::NONE,
            Self::Front => vk::CullModeFlags::FRONT,
            Self::Back => vk::CullModeFlags::BACK,
            Self::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

impl FrontFace {
    pub(crate) const fn to_vk_flag(&self) -> vk::FrontFace {
        match self {
            Self::Clockwise => vk::FrontFace::CLOCKWISE,
            Self::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

impl PolygonMode {
    pub(crate) fn to_vk_flag(&self) -> vk::PolygonMode {
        match self {
            Self::Fill => vk::PolygonMode::FILL,
            Self::Line => vk::PolygonMode::LINE,
            Self::Point => vk::PolygonMode::POINT,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DepthStencilOptions {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: CompareOp,
    pub stencil_test_enable: bool,
}

impl Default for DepthStencilOptions {
    fn default() -> Self {
        Self {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_op: CompareOp::Less,
            stencil_test_enable: false,
        }
    }
}

//Outputs for dynamic rendering
#[derive(Clone)]
pub struct PipelineOutputs {
    pub color: Vec<Format>,
    pub depth: Option<Format>,
    pub stencil: Option<Format>,
}

impl Default for PipelineOutputs {
    fn default() -> Self {
        return PipelineOutputs {
            color: vec![Format::Rgba16Float],
            depth: None,
            stencil: None,
        };
    }
}

#[derive(Clone, Copy)]
pub struct ShaderStages(pub vk::ShaderStageFlags);

impl ShaderStages {
    pub const VERTEX: Self = Self(vk::ShaderStageFlags::VERTEX);
    pub const TESSELLATION_CONTROL: Self = Self(vk::ShaderStageFlags::TESSELLATION_CONTROL);
    pub const TESSELLATION_EVALUATION: Self = Self(vk::ShaderStageFlags::TESSELLATION_EVALUATION);
    pub const GEOMETRY: Self = Self(vk::ShaderStageFlags::GEOMETRY);
    pub const FRAGMENT: Self = Self(vk::ShaderStageFlags::FRAGMENT);
    pub const COMPUTE: Self = Self(vk::ShaderStageFlags::COMPUTE);
    pub const ALL_GRAPHICS: Self = Self(vk::ShaderStageFlags::ALL_GRAPHICS);
    pub const EMPTY: Self = Self(vk::ShaderStageFlags::empty());
    pub const ALL: Self = Self(vk::ShaderStageFlags::ALL);

    pub fn to_vk(self) -> vk::ShaderStageFlags {
        self.0
    }
}

impl BitOr for ShaderStages {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone)]
pub struct PushConstants {
    pub stage_flags: ShaderStages,
    pub offset: u32,
    pub size: u32,
}

impl Default for PushConstants {
    fn default() -> Self {
        return PushConstants {
            stage_flags: ShaderStages::ALL,
            offset: 0,
            size: 0,
        };
    }
}

#[derive(Clone)]
pub struct RasterizationPipelineDescription {
    pub vertex_input: VertexInputDescription,
    pub push_constants: PushConstants,
    pub vertex_shader_path: &'static str,
    pub fragment_shader_path: &'static str,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub polygon_mode: PolygonMode,
    pub depth_stencil: DepthStencilOptions,
    pub alpha_blend_enable: bool,
    pub outputs: PipelineOutputs,
}

impl Default for RasterizationPipelineDescription {
    fn default() -> Self {
        Self {
            vertex_input: VertexInputDescription::default(),
            push_constants: PushConstants::default(),
            vertex_shader_path: " ",
            fragment_shader_path: " ",
            cull_mode: CullMode::None,
            front_face: FrontFace::CounterClockwise,
            polygon_mode: PolygonMode::Fill,
            depth_stencil: DepthStencilOptions::default(),
            alpha_blend_enable: false,
            outputs: PipelineOutputs::default(),
        }
    }
}

//// Compute Pipeline create info ////
pub struct ComputePipelineDescription {
    pub shader_path: &'static str,
    pub push_constants: PushConstants,
}
