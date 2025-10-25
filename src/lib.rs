pub(crate) mod backend;

pub mod core;
pub mod taskgraph;
pub mod utils;

pub use core::{commands::*, definations::*, device::*, gpu_resources::*, instance::*, pipelines::*, swapchain::*};

pub use taskgraph::{definations::*, task_graph::*};

//Macros here
//
// Vertex macro

#[macro_export]
macro_rules! vertex {
    (
        $name:ident {
            input_rate: $rate:ident,
            $( $field:ident : $ty:ty => { location: $loc:expr, format: $fmt:ident } ),* $(,)?
        }
    ) => {
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            $( pub $field: $ty, )*
        }

        impl $name {
            fn vertex_input_description() -> $crate::VertexInputDescription {
                $crate::VertexInputDescription {
                    bindings: vec![
                        ash::vk::VertexInputBindingDescription {
                            binding: 0,
                            stride: std::mem::size_of::<Self>() as u32,
                            input_rate: ash::vk::VertexInputRate::$rate,
                        }
                    ],
                    attributes: vec![
                        $(
                            ash::vk::VertexInputAttributeDescription {
                                location: $loc,
                                binding: 0,
                                format: ash::vk::Format::$fmt,
                                offset: memoffset::offset_of!($name, $field) as u32,
                            }
                        ),*
                    ],
                }
            }
        }
    };
}
