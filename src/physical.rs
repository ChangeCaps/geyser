use std::{ffi, fmt, sync::Arc};

use ash::vk;

use crate::{DeviceExtensions, Instance, InstanceInner, Version, VulkanError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicalDeviceKind {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct QueueFlags: u32 {
        const GRAPHICS           = 0x00000001;
        const COMPUTE            = 0x00000002;
        const TRANSFER           = 0x00000004;
        const SPARSE_BINDING     = 0x00000008;
        const PROTECTED          = 0x00000010;
        const VIDEO_DECODE       = 0x00000020;
        const VIDEO_ENCODE       = 0x00000040;
        const OPTICAL_FLOW       = 0x00000100;
        const DATA_GRAPH_BIT_ARM = 0x00000400;
    }
}

#[derive(Clone, Debug)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
}

#[derive(Clone, Debug)]
pub struct PhysicalDeviceProperties {
    pub api_version: Version,
    pub driver_version: Version,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_kind: PhysicalDeviceKind,
    pub device_name: String,
}

pub struct PhysicalDevice {
    pub(crate) inner: Arc<PhysicalDeviceInner>,
}

impl PhysicalDevice {
    pub fn properties(&self) -> &PhysicalDeviceProperties {
        &self.inner.properties
    }

    pub fn queue_families(&self) -> &[QueueFamilyProperties] {
        &self.inner.queue_family_properties
    }

    pub fn supported_extensions(&self) -> &DeviceExtensions {
        &self.inner.supported_extensions
    }
}

impl fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("handle", &self.inner.handle)
            .field("properties", &self.properties())
            .field("families", &self.queue_families())
            .finish()
    }
}

pub(crate) struct PhysicalDeviceInner {
    pub(crate) handle: vk::PhysicalDevice,

    pub(crate) instance: Arc<InstanceInner>,
    pub(crate) properties: PhysicalDeviceProperties,
    pub(crate) queue_family_properties: Vec<QueueFamilyProperties>,
    pub(crate) supported_extensions: DeviceExtensions,
}

impl Instance {
    pub fn enumerate_physical_devices(&self) -> Vec<PhysicalDevice> {
        self.try_enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    }

    pub fn try_enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, VulkanError> {
        let devices = unsafe { self.inner.handle.enumerate_physical_devices()? };

        devices
            .into_iter()
            .map(|handle| {
                let properties = unsafe {
                    // get the properties of the physical device
                    self.inner.handle.get_physical_device_properties(handle)
                };

                let properties = PhysicalDeviceProperties {
                    api_version: Version::from_vk_version(properties.api_version),
                    driver_version: Version::from_vk_version(properties.driver_version),
                    vendor_id: properties.vendor_id,
                    device_id: properties.device_id,
                    device_kind: match properties.device_type {
                        vk::PhysicalDeviceType::INTEGRATED_GPU => PhysicalDeviceKind::IntegratedGpu,
                        vk::PhysicalDeviceType::DISCRETE_GPU => PhysicalDeviceKind::DiscreteGpu,
                        vk::PhysicalDeviceType::VIRTUAL_GPU => PhysicalDeviceKind::VirtualGpu,
                        vk::PhysicalDeviceType::CPU => PhysicalDeviceKind::Cpu,
                        vk::PhysicalDeviceType::OTHER => PhysicalDeviceKind::Other,
                        _ => PhysicalDeviceKind::Other, // Fallback for unknown types
                    },
                    device_name: unsafe {
                        assert!(properties.device_name.contains(&0));
                        ffi::CStr::from_ptr(properties.device_name.as_ptr())
                    }
                    .to_string_lossy()
                    .into_owned(),
                };

                let family_properties = unsafe {
                    // get the queue family properties of the physical device
                    (self.inner.handle).get_physical_device_queue_family_properties(handle)
                };

                let family_properties = family_properties
                    .into_iter()
                    .map(|family| QueueFamilyProperties {
                        queue_flags: QueueFlags::from_bits_truncate(family.queue_flags.as_raw()),
                        queue_count: family.queue_count,
                    })
                    .collect();

                let extension_properties = unsafe {
                    // get the device extension properties of the physical device
                    (self.inner.handle).enumerate_device_extension_properties(handle)?
                };

                let names = extension_properties.iter().map(|prop| unsafe {
                    ffi::CStr::from_ptr(prop.extension_name.as_ptr())
                        .to_str()
                        .expect("Invalid UTF-8 in extension name")
                });

                let supported_extensions = DeviceExtensions::from_names(names);

                let inner = PhysicalDeviceInner {
                    handle,

                    instance: self.inner.clone(),
                    properties,
                    queue_family_properties: family_properties,
                    supported_extensions,
                };

                Ok(PhysicalDevice {
                    inner: Arc::new(inner),
                })
            })
            .collect::<Result<_, _>>()
    }
}
