use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

//////CORE DESCRIPTIONS//////
#[repr(u32)]
#[derive(Clone)]
pub enum ApiVersion {
    VK_API_1_2 = ash::vk::API_VERSION_1_2,
    VK_API_1_3 = ash::vk::API_VERSION_1_3,
}

pub struct InstanceDescription<W: HasDisplayHandle + HasWindowHandle> {
    pub api_version: ApiVersion,
    pub enable_validation_layers: bool,
    pub window: Arc<W>,
}

pub struct DeviceDescription {
    pub use_compute_queue: bool,
    pub use_transfer_queue: bool,
}

pub struct SwapchainDescription {
    pub image_count: u32,
    pub width: u32,
    pub height: u32,
}

////COMMON MEMORY TYPES////
pub enum MemoryType {
    DEVICE_LOCAL,
    CPU_TO_GPU,
    GPU_TO_CPU,
    AUTO,
}

impl MemoryType {
    pub(crate) const fn to_vk_flag(&self) -> vk_mem::MemoryUsage {
        match self {
            Self::DEVICE_LOCAL => vk_mem::MemoryUsage::AutoPreferDevice,
            Self::CPU_TO_GPU => vk_mem::MemoryUsage::AutoPreferHost,
            Self::GPU_TO_CPU => vk_mem::MemoryUsage::AutoPreferHost, // same hint, but usage differs
            Self::AUTO => vk_mem::MemoryUsage::Auto,
        }
    }
}

///// BUFFER DESCRIPTION /////
#[derive(Clone)]
pub enum BufferUsage {
    STAGING,
    STORAGE,
    VERTEX,
    INDEX,
    UNIFORM,
    INDIRECT,
    TRANSFER_SRC,
    TRANSFER_DST,
}

impl BufferUsage {
    pub(crate) fn to_vk_flag(&self) -> vk::BufferUsageFlags {
        match self {
            Self::STAGING => {
                vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST
            }
            Self::STORAGE => vk::BufferUsageFlags::STORAGE_BUFFER,
            Self::VERTEX => vk::BufferUsageFlags::VERTEX_BUFFER,
            Self::INDEX => vk::BufferUsageFlags::INDEX_BUFFER,
            Self::UNIFORM => vk::BufferUsageFlags::UNIFORM_BUFFER,
            Self::INDIRECT => vk::BufferUsageFlags::INDIRECT_BUFFER,
            Self::TRANSFER_SRC => vk::BufferUsageFlags::TRANSFER_SRC,
            Self::TRANSFER_DST => vk::BufferUsageFlags::TRANSFER_DST,
        }
    }
}

pub struct BufferDescription {
    pub usage: BufferUsage,
    pub size: vk::DeviceSize,
    pub memory_type: MemoryType,
}

impl Default for BufferDescription {
    fn default() -> Self {
        return BufferDescription {
            usage: BufferUsage::STORAGE,
            size: 10,
            memory_type: MemoryType::AUTO,
        };
    }
}

//// IMAGE DESCRIPTION ////
#[derive(Clone, Copy, Debug)]
pub enum ImageType {
    TYPE_1D,
    TYPE_2D,
    TYPE_3D,
}

impl ImageType {
    pub(crate) fn to_vk(&self) -> vk::ImageType {
        match self {
            Self::TYPE_1D => vk::ImageType::TYPE_1D,
            Self::TYPE_2D => vk::ImageType::TYPE_2D,
            Self::TYPE_3D => vk::ImageType::TYPE_3D,
        }
    }
}

#[derive(Clone)]
pub enum ImageUsage {
    TRANSFER_SRC,
    TRANSFER_DST,
    SAMPLED,
    STORAGE,
    COLOR_ATTACHMENT,
    DEPTH_STENCIL_ATTACHMENT,
}

impl ImageUsage {
    pub(crate) const fn to_vk_flag(&self) -> vk::ImageUsageFlags {
        return match self {
            Self::TRANSFER_SRC => vk::ImageUsageFlags::TRANSFER_SRC,
            Self::TRANSFER_DST => vk::ImageUsageFlags::TRANSFER_DST,
            Self::SAMPLED => vk::ImageUsageFlags::SAMPLED,
            Self::STORAGE => vk::ImageUsageFlags::STORAGE,
            Self::COLOR_ATTACHMENT => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            Self::DEPTH_STENCIL_ATTACHMENT => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        };
    }
}

#[derive(Clone)]
pub enum ImageFormat {
    R8G8B8A8_UNORM,
    B8G8R8A8_UNORM,
    R16G16B16A16_SFLOAT,
    D32_SFLOAT,
}

impl ImageFormat {
    pub(crate) const fn to_vk_format(&self) -> vk::Format {
        return match self {
            Self::R8G8B8A8_UNORM => vk::Format::R8G8B8A8_UNORM,
            Self::B8G8R8A8_UNORM => vk::Format::B8G8R8A8_UNORM,
            Self::R16G16B16A16_SFLOAT => vk::Format::R16G16B16A16_SFLOAT,
            Self::D32_SFLOAT => vk::Format::D32_SFLOAT,
        };
    }
}

#[repr(u32)]
pub enum SampleCount {
    TYPE_1,
    TYPE_2,
    TYPE_4,
    TYPE_8,
    TYPE_16,
    TYPE_32,
    TYPE_64,
}

impl SampleCount {
    pub(crate) const fn to_vk_flags(&self) -> vk::SampleCountFlags {
        return match self {
            Self::TYPE_1 => vk::SampleCountFlags::TYPE_1,
            Self::TYPE_2 => vk::SampleCountFlags::TYPE_2,
            Self::TYPE_4 => vk::SampleCountFlags::TYPE_4,
            Self::TYPE_8 => vk::SampleCountFlags::TYPE_8,
            Self::TYPE_16 => vk::SampleCountFlags::TYPE_16,
            Self::TYPE_32 => vk::SampleCountFlags::TYPE_32,
            Self::TYPE_64 => vk::SampleCountFlags::TYPE_64,
        };
    }
}

pub struct ImageDescription {
    pub usage: ImageUsage,
    pub format: ImageFormat,
    pub image_type: ImageType,
    pub height: u32,
    pub width: u32,
    pub depth: u32,
    pub memory_type: MemoryType,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCount,
}

impl Default for ImageDescription {
    fn default() -> Self {
        return Self {
            usage: ImageUsage::SAMPLED,
            format: ImageFormat::R8G8B8A8_UNORM,
            image_type: ImageType::TYPE_2D,
            height: 1,
            width: 1,
            depth: 1,
            memory_type: MemoryType::AUTO,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCount::TYPE_1,
        };
    }
}

//// Image View Description ////
#[derive(Clone, Copy)]
pub enum ImageViewType {
    TYPE_1D,
    TYPE_2D,
    TYPE_3D,
    CUBE,
    TYPE_1D_ARRAY,
    TYPE_2D_ARRAY,
    CUBE_ARRAY,
}

impl ImageViewType {
    pub(crate) const fn to_vk_type(&self) -> vk::ImageViewType {
        match self {
            Self::TYPE_1D => vk::ImageViewType::TYPE_1D,
            Self::TYPE_2D => vk::ImageViewType::TYPE_2D,
            Self::TYPE_3D => vk::ImageViewType::TYPE_3D,
            Self::CUBE => vk::ImageViewType::CUBE,
            Self::TYPE_1D_ARRAY => vk::ImageViewType::TYPE_1D_ARRAY,
            Self::TYPE_2D_ARRAY => vk::ImageViewType::TYPE_2D_ARRAY,
            Self::CUBE_ARRAY => vk::ImageViewType::CUBE_ARRAY,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ImageAspect {
    COLOR,
    DEPTH,
    STENCIL,
    DEPTH_STENCIL,
}

impl ImageAspect {
    pub(crate) fn to_vk_aspect(&self) -> vk::ImageAspectFlags {
        match self {
            Self::COLOR => vk::ImageAspectFlags::COLOR,
            Self::DEPTH => vk::ImageAspectFlags::DEPTH,
            Self::STENCIL => vk::ImageAspectFlags::STENCIL,
            Self::DEPTH_STENCIL => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
        }
    }
}

pub struct ImageViewDescription {
    pub view_type: ImageViewType,
    pub aspect: ImageAspect,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageViewDescription {
    fn default() -> Self {
        return Self {
            view_type: ImageViewType::TYPE_2D,
            aspect: ImageAspect::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
    }
}

//// SAMPLER DESCRIPTION ////
#[derive(Clone, Copy, Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    pub(crate) fn to_vk(self) -> vk::Filter {
        match self {
            Filter::Nearest => vk::Filter::NEAREST,
            Filter::Linear => vk::Filter::LINEAR,
        }
    }
}

/// Mipmap filter mode
#[derive(Clone, Copy, Debug)]
pub enum SamplerMipmapMode {
    Nearest,
    Linear,
}

impl SamplerMipmapMode {
    pub(crate) fn to_vk(self) -> vk::SamplerMipmapMode {
        match self {
            SamplerMipmapMode::Nearest => vk::SamplerMipmapMode::NEAREST,
            SamplerMipmapMode::Linear => vk::SamplerMipmapMode::LINEAR,
        }
    }
}

/// Addressing (wrap/clamp modes)
#[derive(Clone, Copy, Debug)]
pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}
impl SamplerAddressMode {
    pub(crate) fn to_vk(self) -> vk::SamplerAddressMode {
        match self {
            SamplerAddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
            SamplerAddressMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            SamplerAddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            SamplerAddressMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }
}

/// Border colors for ClampToBorder
#[derive(Clone, Copy, Debug)]
pub enum BorderColor {
    FloatTransparentBlack,
    IntTransparentBlack,
    FloatOpaqueBlack,
    IntOpaqueBlack,
    FloatOpaqueWhite,
    IntOpaqueWhite,
}
impl BorderColor {
    pub(crate) fn to_vk(self) -> vk::BorderColor {
        match self {
            BorderColor::FloatTransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
            BorderColor::IntTransparentBlack => vk::BorderColor::INT_TRANSPARENT_BLACK,
            BorderColor::FloatOpaqueBlack => vk::BorderColor::FLOAT_OPAQUE_BLACK,
            BorderColor::IntOpaqueBlack => vk::BorderColor::INT_OPAQUE_BLACK,
            BorderColor::FloatOpaqueWhite => vk::BorderColor::FLOAT_OPAQUE_WHITE,
            BorderColor::IntOpaqueWhite => vk::BorderColor::INT_OPAQUE_WHITE,
        }
    }
}

/// Optional compare operation for depth samplers
#[derive(Clone, Copy, Debug)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}
impl CompareOp {
    pub(crate) fn to_vk(self) -> vk::CompareOp {
        match self {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }
}

pub struct SamplerDescription {
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub max_anisotropy: Option<f32>,
    pub compare_op: Option<CompareOp>,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: bool,
}

impl Default for SamplerDescription {
    fn default() -> Self {
        Self {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            mipmap_mode: SamplerMipmapMode::Linear,
            address_mode_u: SamplerAddressMode::Repeat,
            address_mode_v: SamplerAddressMode::Repeat,
            address_mode_w: SamplerAddressMode::Repeat,
            mip_lod_bias: 0.0,
            max_anisotropy: None,
            compare_op: None,
            min_lod: 0.0,
            max_lod: 1000.0,
            border_color: BorderColor::IntOpaqueBlack,
            unnormalized_coordinates: false,
        }
    }
}

//// TEXTURE DESCRIPTION ////
pub struct TextureDescription {
    pub filter: Filter,
    pub wrap: SamplerAddressMode,
    pub path: &'static str,
    pub generate_mips: bool,
}

impl Default for TextureDescription {
    fn default() -> Self {
        return Self {
            filter: Filter::Linear,
            wrap: SamplerAddressMode::Repeat,
            path: " ",
            generate_mips: true,
        };
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
    pub depth_compare_op: vk::CompareOp,
    pub stencil_test_enable: bool,
}

impl Default for DepthStencilOptions {
    fn default() -> Self {
        Self {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_op: vk::CompareOp::LESS,
            stencil_test_enable: false,
        }
    }
}

//Vertex description for the pipeline
#[derive(Clone)]
pub struct VertexInputDescription {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl Default for VertexInputDescription {
    fn default() -> Self {
        return Self {
            bindings: Vec::new(),
            attributes: Vec::new(),
        };
    }
}

//Outputs for dynamic rendering
#[derive(Clone)]
pub struct PipelineOutputs {
    pub color: Vec<ImageFormat>,
    pub depth: Option<ImageFormat>,
    pub stencil: Option<ImageFormat>,
}

impl Default for PipelineOutputs {
    fn default() -> Self {
        return PipelineOutputs {
            color: vec![ImageFormat::R16G16B16A16_SFLOAT],
            depth: None,
            stencil: None,
        };
    }
}

#[derive(Clone)]
pub struct RasterizationPipelineDescription {
    pub vertex_input: VertexInputDescription,
    pub vertex_shader_path: &'static str,
    pub fragment_shader_path: &'static str,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub polygon_mode: PolygonMode,
    pub line_width: f32,
    pub depth_stencil: DepthStencilOptions,
    pub alpha_blend_enable: bool,
    pub outputs: PipelineOutputs,
}

impl Default for RasterizationPipelineDescription {
    fn default() -> Self {
        Self {
            vertex_input: VertexInputDescription::default(),
            vertex_shader_path: " ",
            fragment_shader_path: " ",
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            polygon_mode: PolygonMode::Fill,
            line_width: 1.0,
            depth_stencil: DepthStencilOptions::default(),
            alpha_blend_enable: false,
            outputs: PipelineOutputs::default(),
        }
    }
}
