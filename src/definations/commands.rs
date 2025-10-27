use ash::vk;
use std::u64;

use crate::*;

use crate::{BufferID, ExecutableCommandBuffer, Fence, ImageID, ImageViewID, Semaphore};

#[derive(Clone, Copy, PartialEq)]
pub enum QueueType {
    Graphics,
    Transfer,
    Compute,
    None,
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
                depth_stencil: vk::ClearDepthStencilValue { depth: *depth, stencil: *stencil },
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
        Self::DepthStencil { depth: 1.0, stencil: 0 }
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
            image_view: ImageViewID { id: u64::max_value() },
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
            Self::ContentsSecondaryCommandBuffers => vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS,
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
            render_area: RenderArea { offset: 0, width: 0, height: 0 },
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
            PipelineStage::ColorAttachmentOutput => vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
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
pub struct MemoryBarrier {
    pub src_stage: PipelineStage,
    pub dst_stage: PipelineStage,
    pub src_access: AccessType,
    pub dst_access: AccessType,
}

impl Default for MemoryBarrier {
    fn default() -> Self {
        return MemoryBarrier {
            src_stage: PipelineStage::TopOfPipe,
            dst_stage: PipelineStage::BottomOfPipe,
            src_access: AccessType::ColorAttachmentRead,
            dst_access: AccessType::ColorAttachmentRead,
        };
    }
}

#[derive(Clone)]
pub struct ImageBarrier {
    pub image: ImageID,
    pub aspect: ImageAspect,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_stage: PipelineStage,
    pub dst_stage: PipelineStage,
    pub src_access: AccessType,
    pub dst_access: AccessType,
    pub src_queue: QueueType,
    pub dst_queue: QueueType,
    pub base_mip: u32,
    pub level_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageBarrier {
    fn default() -> Self {
        return ImageBarrier {
            image: ImageID { id: u64::MAX },
            aspect: ImageAspect::Color,
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::Undefined,
            src_stage: PipelineStage::TopOfPipe,
            dst_stage: PipelineStage::BottomOfPipe,
            src_access: AccessType::ColorAttachmentRead,
            dst_access: AccessType::ColorAttachmentRead,
            src_queue: QueueType::None,
            dst_queue: QueueType::None,
            base_mip: 0,
            level_count: 1,
            base_layer: 0,
            layer_count: 1,
        };
    }
}

#[derive(Clone)]
pub struct BufferBarrier {
    pub buffer: BufferID,
    pub src_stage: PipelineStage,
    pub dst_stage: PipelineStage,
    pub src_access: AccessType,
    pub dst_access: AccessType,
    pub src_queue: QueueType,
    pub dst_queue: QueueType,
    pub offset: u64,
    pub size: u64,
}

impl Default for BufferBarrier {
    fn default() -> Self {
        return BufferBarrier {
            buffer: BufferID { id: u64::MAX },
            src_stage: PipelineStage::TopOfPipe,
            dst_stage: PipelineStage::BottomOfPipe,
            src_access: AccessType::ColorAttachmentRead,
            dst_access: AccessType::ColorAttachmentRead,
            src_queue: QueueType::None,
            dst_queue: QueueType::None,
            offset: 0,
            size: 0,
        };
    }
}

#[derive(Clone)]
pub enum Barrier {
    Memory(MemoryBarrier),
    Image(ImageBarrier),
    Buffer(BufferBarrier),
}

//Submit info
pub struct SemaphoreInfo {
    pub semaphore: Semaphore,
    pub pipeline_stage: PipelineStage,
    pub value: Option<u64>,
}

pub struct QueueSubmitInfo {
    pub fence: Option<Fence>,
    pub command_buffers: Vec<ExecutableCommandBuffer>,
    pub wait_semaphores: Vec<SemaphoreInfo>,
    pub signal_semaphores: Vec<SemaphoreInfo>,
}
