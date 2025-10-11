use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use smallvec::SmallVec;
use std::ops::BitOr;
use std::sync::Arc;

use crate::{BufferID, CommandBuffer, Fence, ImageID, ImageViewID, Semaphore};

//////CORE DESCRIPTIONS//////
#[repr(u32)]
#[derive(Clone)]
pub enum ApiVersion {
    VkApi1_3 = ash::vk::API_VERSION_1_3,
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

#[derive(Clone)]
pub struct SwapchainDescription {
    pub image_count: u32,
    pub width: u32,
    pub height: u32,
}

////COMMON MEMORY TYPES////
pub enum MemoryType {
    DeviceLocal,
    PreferHost,
    Auto,
}

impl MemoryType {
    pub(crate) const fn to_vk_flag(&self) -> vk_mem::MemoryUsage {
        match self {
            Self::DeviceLocal => vk_mem::MemoryUsage::AutoPreferDevice,
            Self::PreferHost => vk_mem::MemoryUsage::AutoPreferHost,
            Self::Auto => vk_mem::MemoryUsage::Auto,
        }
    }
}

///// BUFFER DESCRIPTION /////
pub struct BufferUsage {
    pub(crate) flags: vk::BufferUsageFlags,
}

impl BufferUsage {
    // Define public constants for the common usage types
    pub const STORAGE: Self = Self {
        flags: vk::BufferUsageFlags::STORAGE_BUFFER,
    };
    pub const VERTEX: Self = Self {
        flags: vk::BufferUsageFlags::VERTEX_BUFFER,
    };
    pub const INDEX: Self = Self {
        flags: vk::BufferUsageFlags::INDEX_BUFFER,
    };
    pub const UNIFORM: Self = Self {
        flags: vk::BufferUsageFlags::UNIFORM_BUFFER,
    };
    pub const INDIRECT: Self = Self {
        flags: vk::BufferUsageFlags::INDIRECT_BUFFER,
    };
    pub const TRANSFER_SRC: Self = Self {
        flags: vk::BufferUsageFlags::TRANSFER_SRC,
    };
    pub const TRANSFER_DST: Self = Self {
        flags: vk::BufferUsageFlags::TRANSFER_DST,
    };

    // A method to expose the inner flags for use with ash
    pub(crate) fn to_vk_flag(&self) -> vk::BufferUsageFlags {
        self.flags
    }
}

// Implement the BitOr trait to allow the '|' operator
impl BitOr for BufferUsage {
    type Output = Self;

    // This method defines what happens when you use 'a | b'
    fn bitor(self, other: Self) -> Self::Output {
        Self {
            flags: self.flags | other.flags,
        }
    }
}

// Implement the BitOrAssign trait to allow the '|= ' operator (optional but good practice)
impl BitOr<BufferUsage> for &BufferUsage {
    type Output = BufferUsage;
    fn bitor(self, other: BufferUsage) -> Self::Output {
        BufferUsage {
            flags: self.flags | other.flags,
        }
    }
}

impl BitOr<&BufferUsage> for BufferUsage {
    type Output = BufferUsage;
    fn bitor(self, other: &BufferUsage) -> Self::Output {
        BufferUsage {
            flags: self.flags | other.flags,
        }
    }
}

pub struct BufferDescription {
    pub usage: BufferUsage,
    pub size: vk::DeviceSize,
    pub memory_type: MemoryType,
    pub create_mapped: bool,
}

impl Default for BufferDescription {
    fn default() -> Self {
        return BufferDescription {
            usage: BufferUsage::STORAGE,
            size: 10,
            memory_type: MemoryType::Auto,
            create_mapped: false,
        };
    }
}

//// IMAGE DESCRIPTION ////
#[derive(Clone, Copy, Debug)]
pub enum ImageType {
    Type1D,
    Type2D,
    Type3D,
}

impl ImageType {
    pub(crate) fn to_vk(&self) -> vk::ImageType {
        match self {
            Self::Type1D => vk::ImageType::TYPE_1D,
            Self::Type2D => vk::ImageType::TYPE_2D,
            Self::Type3D => vk::ImageType::TYPE_3D,
        }
    }
}

#[derive(Clone)]
pub enum ImageUsage {
    TransferSrc,
    TransferDst,
    Sampled,
    Storage,
    ColorAttachment,
    DepthStencilAttachment,
}

impl ImageUsage {
    pub(crate) const fn to_vk_flag(&self) -> vk::ImageUsageFlags {
        return match self {
            Self::TransferSrc => vk::ImageUsageFlags::TRANSFER_SRC,
            Self::TransferDst => vk::ImageUsageFlags::TRANSFER_DST,
            Self::Sampled => vk::ImageUsageFlags::SAMPLED,
            Self::Storage => vk::ImageUsageFlags::STORAGE,
            Self::ColorAttachment => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            Self::DepthStencilAttachment => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        };
    }
}
#[derive(Clone)]
pub enum Format {
    // --- Unsigned Normalized (UNORM) Formats - Standard Color & Textures ---
    Rgba8Unorm,  // R8G8B8A8_UNORM (Standard color/texture format)
    Bgra8Unorm,  // B8G8R8A8_UNORM (Common swapchain format)
    Rgb565Unorm, // R5G6B5_UNORM (Low-end texture, 16-bit packed)

    // --- Signed/Unsigned Integers (SINT/UINT) ---
    Rgba8Uint,  // R8G8B8A8_UINT (Used for data buffers/image storage)
    Rgba32Sint, // R32G32B32A32_SINT (Used for data buffers)

    // --- Float Formats (SFLOAT) - High Precision & Data ---
    Rgba16Float, // R16G16B16A16_SFLOAT (HDR/Intermediate targets)
    Rg32Float,   // R32G32_SFLOAT (Used for 2D position or data)
    Rgba32Float, // R32G32B32A32_SFLOAT (Highest precision data storage)
    R32Float,    // R32_SFLOAT (Single channel float data)

    // --- Depth and Stencil Formats ---
    D32Float,       // D32_SFLOAT (32-bit depth-only)
    D24UnormS8Uint, // D24_UNORM_S8_UINT (Most common combined Depth/Stencil)
    D16Unorm,       // D16_UNORM (16-bit depth-only)

    // --- Block Compressed (Slightly less common but essential for assets) ---
    BC1RgbaUnorm, // BC1_RGBA_UNORM_BLOCK (DXT1 - Low quality compression)
    BC7Unorm,     // BC7_UNORM_BLOCK (High quality compression)
}

impl Format {
    pub(crate) const fn to_vk_format(&self) -> vk::Format {
        return match self {
            // Unsigned Normalized (UNORM)
            Self::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
            Self::Bgra8Unorm => vk::Format::B8G8R8A8_UNORM,
            Self::Rgb565Unorm => vk::Format::R5G6B5_UNORM_PACK16,

            // Signed/Unsigned Integers (SINT/UINT)
            Self::Rgba8Uint => vk::Format::R8G8B8A8_UINT,
            Self::Rgba32Sint => vk::Format::R32G32B32A32_SINT,

            // Float Formats (SFLOAT)
            Self::Rgba16Float => vk::Format::R16G16B16A16_SFLOAT,
            Self::Rg32Float => vk::Format::R32G32_SFLOAT,
            Self::Rgba32Float => vk::Format::R32G32B32A32_SFLOAT,
            Self::R32Float => vk::Format::R32_SFLOAT,

            // Depth and Stencil
            Self::D32Float => vk::Format::D32_SFLOAT,
            Self::D24UnormS8Uint => vk::Format::D24_UNORM_S8_UINT,
            Self::D16Unorm => vk::Format::D16_UNORM,

            // Block Compressed
            Self::BC1RgbaUnorm => vk::Format::BC1_RGBA_UNORM_BLOCK,
            Self::BC7Unorm => vk::Format::BC7_UNORM_BLOCK,
        };
    }
}

#[repr(u32)]
pub enum SampleCount {
    Type1,
    Type2,
    Type4,
    Type8,
    Type16,
    Type32,
    Type64,
}

impl SampleCount {
    pub(crate) const fn to_vk_flags(&self) -> vk::SampleCountFlags {
        return match self {
            Self::Type1 => vk::SampleCountFlags::TYPE_1,
            Self::Type2 => vk::SampleCountFlags::TYPE_2,
            Self::Type4 => vk::SampleCountFlags::TYPE_4,
            Self::Type8 => vk::SampleCountFlags::TYPE_8,
            Self::Type16 => vk::SampleCountFlags::TYPE_16,
            Self::Type32 => vk::SampleCountFlags::TYPE_32,
            Self::Type64 => vk::SampleCountFlags::TYPE_64,
        };
    }
}
#[derive(Copy, Clone, PartialEq)]
pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachment,
    DepthStencilAttachment,
    DepthStencilReadOnly,
    ShaderReadOnly,
    TransferSrc,
    TransferDst,
    PresentSrc,
}

impl ImageLayout {
    #[inline]
    pub(crate) const fn to_vk_layout(self) -> vk::ImageLayout {
        match self {
            ImageLayout::Undefined => vk::ImageLayout::UNDEFINED,
            ImageLayout::General => vk::ImageLayout::GENERAL,
            ImageLayout::ColorAttachment => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilAttachment => {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
            ImageLayout::DepthStencilReadOnly => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            ImageLayout::ShaderReadOnly => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::TransferSrc => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDst => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::PresentSrc => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

pub struct ImageDescription {
    pub usage: ImageUsage,
    pub format: Format,
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
            usage: ImageUsage::Sampled,
            format: Format::Rgba16Float,
            image_type: ImageType::Type2D,
            height: 1,
            width: 1,
            depth: 1,
            memory_type: MemoryType::Auto,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCount::Type1,
        };
    }
}

//// Image View Description ////
#[derive(Clone, Copy)]
pub enum ImageViewType {
    Type1D,
    Type2D,
    Type3D,
    Cube,
    Type1DArray,
    Type2DArray,
    CubeArray,
}

impl ImageViewType {
    pub(crate) const fn to_vk_type(&self) -> vk::ImageViewType {
        match self {
            Self::Type1D => vk::ImageViewType::TYPE_1D,
            Self::Type2D => vk::ImageViewType::TYPE_2D,
            Self::Type3D => vk::ImageViewType::TYPE_3D,
            Self::Cube => vk::ImageViewType::CUBE,
            Self::Type1DArray => vk::ImageViewType::TYPE_1D_ARRAY,
            Self::Type2DArray => vk::ImageViewType::TYPE_2D_ARRAY,
            Self::CubeArray => vk::ImageViewType::CUBE_ARRAY,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ImageAspect {
    Color,
    Depth,
    Stencil,
    DepthStencil,
}

impl ImageAspect {
    pub(crate) fn to_vk_aspect(&self) -> vk::ImageAspectFlags {
        match self {
            Self::Color => vk::ImageAspectFlags::COLOR,
            Self::Depth => vk::ImageAspectFlags::DEPTH,
            Self::Stencil => vk::ImageAspectFlags::STENCIL,
            Self::DepthStencil => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
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
            view_type: ImageViewType::Type2D,
            aspect: ImageAspect::Color,
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

//// Command Pools and Command Buffers ////
#[derive(Clone, Copy, PartialEq)]
pub enum QueueType {
    Graphics,
    Transfer,
    Compute,
}

pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

impl CommandBufferLevel {
    pub(crate) const fn to_vk_flags(&self) -> vk::CommandBufferLevel {
        match self {
            Self::Primary => vk::CommandBufferLevel::PRIMARY,
            Self::Secondary => vk::CommandBufferLevel::SECONDARY,
        }
    }
}

pub enum CommandBufferUsage {
    OneTimeSubmit,
    RenderPassContinue,
    SimultaneousUse,
}

impl CommandBufferUsage {
    pub(crate) const fn to_vk_flags(&self) -> vk::CommandBufferUsageFlags {
        match self {
            Self::OneTimeSubmit => vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            Self::RenderPassContinue => vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
            Self::SimultaneousUse => vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        }
    }
}

pub enum IndexType {
    Uint32,
    Uint16,
}

impl IndexType {
    pub(crate) const fn to_vk_flag(&self) -> vk::IndexType {
        match self {
            Self::Uint32 => vk::IndexType::UINT32,
            Self::Uint16 => vk::IndexType::UINT16,
        }
    }
}

// Render begin info
#[derive(Clone, Copy)]
pub struct RenderArea {
    pub offset: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Copy, Clone, PartialEq)]
pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

impl LoadOp {
    #[inline]
    pub(crate) const fn to_vk(&self) -> vk::AttachmentLoadOp {
        match self {
            Self::Load => vk::AttachmentLoadOp::LOAD,
            Self::Clear => vk::AttachmentLoadOp::CLEAR,
            Self::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare,
    None,
}

impl StoreOp {
    #[inline]
    pub(crate) const fn to_vk(&self) -> vk::AttachmentStoreOp {
        match self {
            Self::Store => vk::AttachmentStoreOp::STORE,
            Self::DontCare => vk::AttachmentStoreOp::DONT_CARE,
            Self::None => vk::AttachmentStoreOp::NONE,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResolveMode {
    None,
    SampleZero,
    Average,
    Min,
    Max,
}

impl ResolveMode {
    #[inline]
    pub(crate) const fn to_vk(&self) -> vk::ResolveModeFlags {
        match self {
            Self::None => vk::ResolveModeFlags::NONE,
            Self::SampleZero => vk::ResolveModeFlags::SAMPLE_ZERO,
            Self::Average => vk::ResolveModeFlags::AVERAGE,
            Self::Min => vk::ResolveModeFlags::MIN,
            Self::Max => vk::ResolveModeFlags::MAX,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClearValue {
    ColorFloat([f32; 4]),
    ColorInt([i32; 4]),
    ColorUint([u32; 4]),
    DepthStencil { depth: f32, stencil: u32 },
}

impl ClearValue {
    #[inline]
    pub(crate) const fn to_vk(&self) -> vk::ClearValue {
        match self {
            Self::ColorFloat(v) => vk::ClearValue {
                color: vk::ClearColorValue { float32: *v },
            },
            Self::ColorInt(v) => vk::ClearValue {
                color: vk::ClearColorValue { int32: *v },
            },
            Self::ColorUint(v) => vk::ClearValue {
                color: vk::ClearColorValue { uint32: *v },
            },
            Self::DepthStencil { depth, stencil } => vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: *depth,
                    stencil: *stencil,
                },
            },
        }
    }

    /// Common helper for zero clear
    pub const fn black() -> Self {
        Self::ColorFloat([0.0, 0.0, 0.0, 1.0])
    }

    /// Common helper for white clear
    pub const fn white() -> Self {
        Self::ColorFloat([1.0, 1.0, 1.0, 1.0])
    }

    /// Common helper for depth clear
    pub const fn depth_one() -> Self {
        Self::DepthStencil {
            depth: 1.0,
            stencil: 0,
        }
    }
}

pub struct RenderingAttachment {
    pub image_view: ImageViewID,
    pub image_layout: ImageLayout,
    pub resolve_mode: ResolveMode,
    pub resolve_image_view: Option<ImageViewID>,
    pub resolve_image_layout: ImageLayout,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: ClearValue,
}

impl Default for RenderingAttachment {
    fn default() -> Self {
        Self {
            image_view: ImageViewID {
                id: u64::max_value(),
            },
            image_layout: ImageLayout::Undefined,
            resolve_image_view: None,
            resolve_image_layout: ImageLayout::Undefined,
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            resolve_mode: ResolveMode::None,
            clear_value: ClearValue::ColorFloat([0.0, 0.0, 0.0, 0.0]),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum RenderingFlags {
    None,
    ContentsSecondaryCommandBuffers,
    Suspending,
    Resuming,
}

impl RenderingFlags {
    #[inline]
    pub(crate) const fn to_vk(&self) -> vk::RenderingFlags {
        match self {
            Self::None => vk::RenderingFlags::empty(),
            Self::ContentsSecondaryCommandBuffers => {
                vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS
            }
            Self::Suspending => vk::RenderingFlags::SUSPENDING,
            Self::Resuming => vk::RenderingFlags::RESUMING,
        }
    }
}

pub struct RenderingBeginInfo {
    pub render_area: RenderArea,
    pub rendering_flags: RenderingFlags,
    pub view_mask: u32,
    pub layer_count: u32,
    pub color_attachments: Vec<RenderingAttachment>,
    pub depth_attachment: Option<RenderingAttachment>,
    pub stencil_attachment: Option<RenderingAttachment>,
}

impl Default for RenderingBeginInfo {
    fn default() -> Self {
        return Self {
            render_area: RenderArea {
                offset: 0,
                width: 0,
                height: 0,
            },
            rendering_flags: RenderingFlags::None,
            view_mask: 0,
            layer_count: 0,
            color_attachments: Vec::new(),
            depth_attachment: None,
            stencil_attachment: None,
        };
    }
}

// Copy commands
pub struct BufferCopyInfo {
    pub src_buffer: BufferID,
    pub dst_buffer: BufferID,
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

// Memory barriers
#[derive(Clone, Copy, Debug)]
pub enum PipelineStage {
    TopOfPipe,
    BottomOfPipe,
    VertexShader,
    FragmentShader,
    ComputeShader,
    ColorAttachmentOutput,
    Transfer,
    AllCommands,
}

impl PipelineStage {
    pub const fn to_vk(&self) -> vk::PipelineStageFlags2 {
        match self {
            PipelineStage::TopOfPipe => vk::PipelineStageFlags2::TOP_OF_PIPE,
            PipelineStage::BottomOfPipe => vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
            PipelineStage::VertexShader => vk::PipelineStageFlags2::VERTEX_SHADER,
            PipelineStage::FragmentShader => vk::PipelineStageFlags2::FRAGMENT_SHADER,
            PipelineStage::ComputeShader => vk::PipelineStageFlags2::COMPUTE_SHADER,
            PipelineStage::ColorAttachmentOutput => {
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
            }
            PipelineStage::Transfer => vk::PipelineStageFlags2::TRANSFER,
            PipelineStage::AllCommands => vk::PipelineStageFlags2::ALL_COMMANDS,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AccessType {
    None,
    Indirect,
    IndexRead,
    VertexRead,
    UniformRead,
    ShaderRead,
    ShaderWrite,
    ColorAttachmentRead,
    ColorAttachmentWrite,
    DepthStencilRead,
    DepthStencilWrite,
    TransferRead,
    TransferWrite,
}

impl AccessType {
    pub const fn to_vk(&self) -> vk::AccessFlags2 {
        match self {
            AccessType::None => vk::AccessFlags2::empty(),
            AccessType::Indirect => vk::AccessFlags2::INDIRECT_COMMAND_READ,
            AccessType::IndexRead => vk::AccessFlags2::INDEX_READ,
            AccessType::VertexRead => vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
            AccessType::UniformRead => vk::AccessFlags2::UNIFORM_READ,
            AccessType::ShaderRead => vk::AccessFlags2::SHADER_READ,
            AccessType::ShaderWrite => vk::AccessFlags2::SHADER_WRITE,
            AccessType::ColorAttachmentRead => vk::AccessFlags2::COLOR_ATTACHMENT_READ,
            AccessType::ColorAttachmentWrite => vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            AccessType::DepthStencilRead => vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
            AccessType::DepthStencilWrite => vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            AccessType::TransferRead => vk::AccessFlags2::TRANSFER_READ,
            AccessType::TransferWrite => vk::AccessFlags2::TRANSFER_WRITE,
        }
    }
}

#[derive(Clone)]
pub enum Barrier {
    Memory {
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_access: AccessType,
        dst_access: AccessType,
    },
    Image {
        image: ImageID,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_access: AccessType,
        dst_access: AccessType,
        base_mip: u32,
        level_count: u32,
        base_layer: u32,
        layer_count: u32,
    },
    Buffer {
        buffer: BufferID,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_access: AccessType,
        dst_access: AccessType,
        offset: u64,
        size: u64,
    },
}

//Submit info
pub struct SemaphoreInfo {
    pub semaphore: Semaphore,
    pub pipeline_stage: PipelineStage,
    pub value: Option<u64>,
}

pub struct QueueSubmitInfo {
    pub fence: Option<Fence>,
    pub command_buffers: SmallVec<[CommandBuffer; 2]>,
    pub wait_semaphores: SmallVec<[SemaphoreInfo; 2]>,
    pub signal_semaphores: SmallVec<[SemaphoreInfo; 2]>,
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

#[derive(Clone)]
pub struct RasterizationPipelineDescription {
    pub vertex_input: VertexInputDescription,
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
            vertex_shader_path: " ",
            fragment_shader_path: " ",
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            polygon_mode: PolygonMode::Fill,
            depth_stencil: DepthStencilOptions::default(),
            alpha_blend_enable: false,
            outputs: PipelineOutputs::default(),
        }
    }
}
