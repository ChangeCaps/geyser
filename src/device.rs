use std::{ffi, fmt, sync::Arc};

use ash::vk;

use crate::{
    InstanceExtensions, InstanceInner, PhysicalDevice, Queue, QueueInner, Requires, Validated,
    ValidationError, Version, VulkanError, is_validation_enabled,
};

include!(concat!(env!("OUT_DIR"), "/device_extensions.rs"));

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DeviceFeatures {
    pub acceleration_structure: bool,
}

#[derive(Clone, Debug)]
pub struct QueueFamilyDescriptor<'a> {
    pub family_index: u32,
    pub priorities: &'a [f32],
}

#[derive(Clone, Debug, Default)]
pub struct DeviceDescriptor<'a> {
    pub queue_families: &'a [QueueFamilyDescriptor<'a>],
    pub enabled_extensions: DeviceExtensions,
    pub enabled_features: DeviceFeatures,
}

pub struct Device {
    pub(crate) inner: Arc<DeviceInner>,
}

impl Device {
    pub fn physical(&self) -> &PhysicalDevice {
        &self.inner.physical
    }

    pub fn enabled_extensions(&self) -> &DeviceExtensions {
        &self.inner.enabled_extensions
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.inner.handle.handle())
            .finish()
    }
}

pub(crate) struct DeviceInner {
    pub(crate) handle: ash::Device,

    pub(crate) physical: PhysicalDevice,

    pub(crate) enabled_extensions: DeviceExtensions,
}

impl DeviceInner {
    pub(crate) fn instance(&self) -> &Arc<InstanceInner> {
        &self.physical.inner.instance
    }
}

impl Drop for DeviceInner {
    fn drop(&mut self) {
        unsafe {
            let _ = self.handle.device_wait_idle();

            tracing::trace!(
                handle = ?self.handle.handle(),
                "Destroying Vulkan device"
            );

            self.handle.destroy_device(None);
        }
    }
}

impl PhysicalDevice {
    #[track_caller]
    pub fn create_device(&self, desc: &DeviceDescriptor<'_>) -> (Device, Vec<Vec<Queue>>) {
        self.try_create_device(desc)
            .expect("Failed to create device")
    }

    pub fn try_create_device(
        &self,
        desc: &DeviceDescriptor<'_>,
    ) -> Result<(Device, Vec<Vec<Queue>>), Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_create_device(desc)?;
        }

        unsafe { self.try_create_device_unchecked(desc).map_err(From::from) }
    }

    /// # Safety
    /// - The 'queue_families' must not be empty.
    /// - The `family_index` in each `QueueFamilyDescriptor` must be unique.
    /// - The `family_index` in each `QueueFamilyDescriptor` must be a valid queue family index for the physical device.
    /// - The `priorities` in each `QueueFamilyDescriptor` must contain only valid queue priorities (between 0.0 and 1.0).
    /// - The `priorities` in each `QueueFamilyDescriptor` must not exceed the number of queues in the family.
    /// - All required extensions for each enabled extension must also be enabled.
    pub unsafe fn try_create_device_unchecked(
        &self,
        desc: &DeviceDescriptor<'_>,
    ) -> Result<(Device, Vec<Vec<Queue>>), VulkanError> {
        let mut queue_create_infos = Vec::with_capacity(desc.queue_families.len());

        for family in desc.queue_families {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: family.family_index,
                queue_count: family.priorities.len() as u32,
                p_queue_priorities: family.priorities.as_ptr(),
                ..Default::default()
            };

            queue_create_infos.push(queue_create_info);
        }

        let enabled_extensions = desc.enabled_extensions.extension_names();
        let enabled_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            p_enabled_features: &enabled_features,
            ..Default::default()
        };

        let handle = unsafe {
            (self.inner.instance.handle).create_device(self.inner.handle, &create_info, None)?
        };

        let inner = DeviceInner {
            handle,

            physical: Self {
                inner: self.inner.clone(),
            },

            enabled_extensions: desc.enabled_extensions.clone(),
        };

        let device = Device {
            inner: Arc::new(inner),
        };

        let mut families = Vec::new();

        for family in desc.queue_families {
            let mut queues = Vec::new();

            for queue_index in 0..family.priorities.len() as u32 {
                let handle = unsafe {
                    (device.inner.handle).get_device_queue(family.family_index, queue_index)
                };

                let inner = QueueInner {
                    handle,

                    device: device.inner.clone(),

                    family_index: family.family_index,
                    queue_index,
                };

                let queue = Queue {
                    inner: Arc::new(inner),
                };

                queues.push(queue);
            }

            families.push(queues);
        }

        Ok((device, families))
    }

    fn validate_create_device(&self, desc: &DeviceDescriptor<'_>) -> Result<(), ValidationError> {
        desc.enabled_extensions.validate(
            &self.inner.instance.enabled_extensions,
            self.inner.instance.api_version,
        )?;

        if desc.queue_families.is_empty() {
            return Err(ValidationError {
                context: "desc.queue_families".into(),
                problem: "At least one queue family must be specified.".into(),
                vuids: &["VUID-VkDeviceCreateInfo-None-10778"],
                ..Default::default()
            });
        }

        for (i, family) in desc.queue_families.iter().enumerate() {
            if family.family_index >= self.queue_families().len() as u32 {
                return Err(ValidationError {
                    context: "desc.queue_families[_].family_index".into(),
                    problem: "Queue family index is out of bounds for the physical device.".into(),
                    vuids: &["VUID-VkDeviceQueueCreateInfo-queueFamilyIndex-00381"],
                    ..Default::default()
                });
            }

            if family.priorities.is_empty() {
                return Err(ValidationError {
                    context: "desc.queue_families[_].priorities".into(),
                    problem: "At least one queue priority must be specified.".into(),
                    vuids: &["VUID-VkDeviceQueueCreateInfo-queueCount-arraylength"],
                    ..Default::default()
                });
            }

            let queue_count =
                self.queue_families()[family.family_index as usize].queue_count as usize;
            let count_out_of_bounds = family.priorities.len() > queue_count;

            if count_out_of_bounds {
                return Err(ValidationError {
                    context: "desc.queue_families[_].priorities".into(),
                    problem:
                        "Number of queue priorities exceeds the number of queues in the family."
                            .into(),
                    vuids: &["VUID-VkDeviceQueueCreateInfo-queueCount-00382"],
                    ..Default::default()
                });
            }

            let has_duplicate = desc.queue_families[..i]
                .iter()
                .any(|f| f.family_index == family.family_index);

            if has_duplicate {
                return Err(ValidationError {
                    context: "desc.queue_families[_].family_index".into(),
                    problem: "Queue family index must be unique.".into(),
                    vuids: &["VUID-VkDeviceCreateInfo-queueFamilyIndex-02802"],
                    ..Default::default()
                });
            }

            let priorities_out_of_bounds = family
                .priorities
                .iter()
                .any(|&p| !(0.0..=1.0).contains(&p) || p.is_nan());

            if priorities_out_of_bounds {
                return Err(ValidationError {
                    context: "desc.queue_families[_].priorities".into(),
                    problem: "Queue priorities must be between 0.0 and 1.0.".into(),
                    vuids: &["VUID-VkDeviceQueueCreateInfo-pQueuePriorities-00383"],
                    ..Default::default()
                });
            }
        }

        Ok(())
    }
}
