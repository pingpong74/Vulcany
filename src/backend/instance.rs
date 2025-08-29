use super::device::Device;

use crate::core::context::{DeviceDescription, InstanceDescription};

use ash;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::{ffi::CStr, os::unix::raw, sync::Arc};

pub(crate) struct Surface {
    handle: ash::vk::SurfaceKHR,
    loader: ash::khr::surface::Instance,
}

pub(crate) struct SwapchainSupport {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>,
}

pub(crate) struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    presetation_family: Option<u32>,
    transfer_family: Option<u32>,
    compute_family: Option<u32>,
}

pub(crate) struct PhysicalDevice {
    handle: ash::vk::PhysicalDevice,
    swapchain_support: SwapchainSupport,
    queue_families: QueueFamilyIndices,
}

pub(crate) struct Instance {
    entry: ash::Entry,
    handle: ash::Instance,
    surface: Surface,
    physical_device_extensions: Vec<&'static CStr>,
}

impl Instance {
    pub(crate) fn new<W: HasDisplayHandle + HasWindowHandle>(
        instance_create_info: &InstanceDescription<W>,
    ) -> Instance {
        let entry = ash::Entry::linked();

        let mut required_extensions = vec![ash::khr::surface::NAME.as_ptr()];

        let raw_window_handle = instance_create_info
            .window
            .window_handle()
            .expect("Failed to accuqire raw window handle")
            .as_raw();

        match raw_window_handle {
            //Windows
            raw_window_handle::RawWindowHandle::Win32(h) => {
                required_extensions.push(ash::khr::win32_surface::NAME.as_ptr());
            }

            //Wayland
            raw_window_handle::RawWindowHandle::Wayland(w) => {
                required_extensions.push(ash::khr::wayland_surface::NAME.as_ptr());
            }

            //Xcb
            raw_window_handle::RawWindowHandle::Xcb(w) => {
                required_extensions.push(ash::khr::xcb_surface::NAME.as_ptr());
            }

            //Apple
            raw_window_handle::RawWindowHandle::AppKit(w) => {
                required_extensions.push(ash::ext::metal_surface::NAME.as_ptr());
            }

            //Panic if none found :(
            _ => {}
        };

        if instance_create_info.enable_validation_layers {
            required_extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let app_info = ash::vk::ApplicationInfo {
            api_version: instance_create_info.api_version.clone() as u32,
            ..Default::default()
        };

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&required_extensions);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        let surface =
            unsafe { Instance::create_surface(&entry, &instance, &instance_create_info.window) };

        return Instance {
            entry: entry,
            handle: instance,
            surface: surface,
            physical_device_extensions: vec![ash::khr::swapchain::NAME],
        };
    }

    pub(crate) fn create_device(&self, device_create_info: &DeviceDescription) {
        let physical_device = {
            let dev = self.select_physical_device();
            if dev.is_none() {
                panic!("Failed to find vulkan compatible device")
            }

            dev
        };
    }
}

//Private functions
impl Instance {
    unsafe fn create_surface<W: HasDisplayHandle + HasWindowHandle>(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Arc<W>,
    ) -> Surface {
        let raw_window_handle = window
            .window_handle()
            .expect("Failed to accuqire raw window handle")
            .as_raw();

        let surface_handle = match raw_window_handle {
            //Windows
            raw_window_handle::RawWindowHandle::Win32(h) => {
                let info = ash::vk::Win32SurfaceCreateInfoKHR {
                    ..Default::default()
                };
                let loader = ash::khr::win32_surface::Instance::new(entry, instance);
                unsafe {
                    loader
                        .create_win32_surface(&info, None)
                        .expect("Failed to create surface")
                }
            }

            //Wayland
            raw_window_handle::RawWindowHandle::Wayland(w) => {
                let info = ash::vk::WaylandSurfaceCreateInfoKHR {
                    ..Default::default()
                };
                println!("HERE!!");
                let loader = ash::khr::wayland_surface::Instance::new(entry, instance);
                unsafe {
                    loader
                        .create_wayland_surface(&info, None)
                        .expect("Failed to create surface")
                }
            }

            //Xcb
            raw_window_handle::RawWindowHandle::Xcb(w) => {
                let info = ash::vk::XcbSurfaceCreateInfoKHR {
                    ..Default::default()
                };
                let loader = ash::khr::xcb_surface::Instance::new(entry, instance);
                unsafe {
                    loader
                        .create_xcb_surface(&info, None)
                        .expect("Failed to create surface")
                }
            }

            //Apple
            raw_window_handle::RawWindowHandle::AppKit(w) => {
                let info = ash::vk::MetalSurfaceCreateInfoEXT {
                    ..Default::default()
                };
                let loader = ash::ext::metal_surface::Instance::new(entry, instance);
                unsafe {
                    loader
                        .create_metal_surface(&info, None)
                        .expect("Failed to create surface")
                }
            }

            //Panic if none found :(
            _ => {
                panic!("Ooo")
            }
        };

        return Surface {
            handle: surface_handle,
            loader: ash::khr::surface::Instance::new(entry, instance),
        };
    }

    fn get_queue_families(
        &self,
        physical_device: ash::vk::PhysicalDevice,
    ) -> Option<QueueFamilyIndices> {
        let queue_families = unsafe {
            self.handle
                .get_physical_device_queue_family_properties(physical_device)
        };

        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            transfer_family: None,
            compute_family: None,
            presetation_family: None,
        };

        for (i, family) in queue_families.iter().enumerate() {
            // Graphics
            if family.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
                && indices.graphics_family.is_none()
            {
                indices.graphics_family = Some(i as u32);
            }

            // Compute (dedicated if possible)
            if family.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
                && indices.compute_family.is_none()
            {
                if !family.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
                    indices.compute_family = Some(i as u32);
                }
            }

            // Transfer (dedicated if possible)
            if family.queue_flags.contains(ash::vk::QueueFlags::TRANSFER)
                && indices.transfer_family.is_none()
            {
                if !family.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
                    && !family.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
                {
                    indices.transfer_family = Some(i as u32);
                }
            }

            // Presentation
            let present_support = unsafe {
                self.surface
                    .loader
                    .get_physical_device_surface_support(
                        physical_device,
                        i as u32,
                        self.surface.handle,
                    )
                    .unwrap()
            };
            if present_support && indices.presetation_family.is_none() {
                indices.presetation_family = Some(i as u32);
            }
        }

        if indices.graphics_family.is_some() && indices.presetation_family.is_some() {
            Some(indices)
        } else {
            None
        }
    }

    fn get_swapchain_support(
        &self,
        physical_device: ash::vk::PhysicalDevice,
    ) -> Option<SwapchainSupport> {
        unsafe {
            let capabilities = self
                .surface
                .loader
                .get_physical_device_surface_capabilities(physical_device, self.surface.handle)
                .ok()?;

            let formats = self
                .surface
                .loader
                .get_physical_device_surface_formats(physical_device, self.surface.handle)
                .ok()?;

            let present_modes = self
                .surface
                .loader
                .get_physical_device_surface_present_modes(physical_device, self.surface.handle)
                .ok()?;

            if formats.is_empty() || present_modes.is_empty() {
                return None;
            } else {
                return Some(SwapchainSupport {
                    capabilities,
                    formats,
                    present_modes,
                });
            }
        }
    }

    fn check_device_extension_support(&self, device: ash::vk::PhysicalDevice) -> bool {
        let available_extensions = unsafe {
            self.handle
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extensions")
        };

        let available_extension_names: Vec<&std::ffi::CStr> = available_extensions
            .iter()
            .map(|ext| {
                // Convert raw `extension_name` to CStr
                let raw_name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                raw_name
            })
            .collect();

        // Check all required extensions are present
        self.physical_device_extensions.iter().all(|&required| {
            available_extension_names
                .iter()
                .any(|&avail| avail == required)
        })
    }

    fn select_physical_device(&self) -> Option<PhysicalDevice> {
        let devices = unsafe {
            self.handle
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        let mut best_device: Option<(i32, PhysicalDevice)> = None;

        for device in devices {
            let props = unsafe { self.handle.get_physical_device_properties(device) };

            if let (Some(qf), Some(sc)) = (
                self.get_queue_families(device),
                self.get_swapchain_support(device),
            ) {
                if !self.check_device_extension_support(device) {
                    continue;
                }

                // Score device: discrete = 1000, integrated = 100, others = 10
                let score = match props.device_type {
                    ash::vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
                    ash::vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
                    _ => 10,
                };

                // Prefer larger max image dimension as tiebreaker
                let score = score + props.limits.max_image_dimension2_d as i32;

                let candidate = PhysicalDevice {
                    handle: device,
                    swapchain_support: sc,
                    queue_families: qf,
                };

                if let Some((best_score, _)) = &best_device {
                    if score > *best_score {
                        best_device = Some((score, candidate));
                    }
                } else {
                    best_device = Some((score, candidate));
                }
            }
        }

        return best_device.map(|(_, dev)| dev);
    }
}

//Drop implementation
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.surface
                .loader
                .destroy_surface(self.surface.handle, None);

            self.handle.destroy_instance(None);
        };
    }
}
