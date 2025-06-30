use std::{borrow::Cow, ffi, fmt, ops::Deref, ptr, sync::Arc};

use ash::vk;

use crate::{Instance, MemoryHeap, MemoryProperties, MemoryType, Queue, RawInstance, Version};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceExtension([ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]);

impl DeviceExtension {
    pub const KHR_16BIT_STORAGE: Self = Self::new(vk::KHR_16BIT_STORAGE_NAME);
    pub const KHR_8BIT_STORAGE: Self = Self::new(vk::KHR_8BIT_STORAGE_NAME);
    pub const KHR_ACCELERATION_STRUCTURE: Self = Self::new(vk::KHR_ACCELERATION_STRUCTURE_NAME);
    pub const KHR_BUFFER_DEVICE_ADDRESS: Self = Self::new(vk::KHR_BUFFER_DEVICE_ADDRESS_NAME);
    pub const KHR_DEFERRED_HOST_OPERATIONS: Self = Self::new(vk::KHR_DEFERRED_HOST_OPERATIONS_NAME);
    pub const KHR_DYNAMIC_RENDERING: Self = Self::new(vk::KHR_DYNAMIC_RENDERING_NAME);
    pub const KHR_RAY_TRACING_PIPELINE: Self = Self::new(vk::KHR_RAY_TRACING_PIPELINE_NAME);
    pub const KHR_SHADER_FLOAT_CONTROLS: Self = Self::new(vk::KHR_SHADER_FLOAT_CONTROLS_NAME);
    pub const KHR_SWAPCHAIN: Self = Self::new(vk::KHR_SWAPCHAIN_NAME);

    pub const EXT_DESCRIPTOR_INDEXING: Self = Self::new(vk::EXT_DESCRIPTOR_INDEXING_NAME);

    /// Create a new device extension from a static C string.
    ///
    /// `Note:` The length of the c string including the null terminator must not exceed
    /// `vk::MAX_EXTENSION_NAME_SIZE`.
    pub const fn new(name: &ffi::CStr) -> Self {
        unsafe { Self::from_raw(name.as_ptr()) }
    }

    /// # Safety
    /// - `extension` must be a valid C string pointer.
    pub const unsafe fn from_raw(extension: *const ffi::c_char) -> Self {
        unsafe {
            let len = ffi::CStr::from_ptr(extension).count_bytes() + 1;
            assert!(len <= vk::MAX_EXTENSION_NAME_SIZE);

            let mut array = [0; vk::MAX_EXTENSION_NAME_SIZE];
            ptr::copy_nonoverlapping(extension, array.as_mut_ptr(), len);

            Self(array)
        }
    }

    pub fn as_c_str(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(self.0.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.0.as_ptr()
    }
}

impl fmt::Debug for DeviceExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DeviceExtension")
            .field(&self.as_c_str().to_string_lossy())
            .finish()
    }
}

impl fmt::Display for DeviceExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_c_str().to_string_lossy())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceExtensionProperties {
    pub name: DeviceExtension,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct QueueDescriptor<'a> {
    pub family: u32,
    pub priorities: &'a [f32],
}

impl<'a> QueueDescriptor<'a> {
    pub const fn new(family: u32, priorities: &'a [f32]) -> Self {
        Self { family, priorities }
    }
}

#[derive(Debug, Default)]
pub struct DeviceDescriptor<'a> {
    pub queues: &'a [QueueDescriptor<'a>],
    pub extensions: &'a [DeviceExtension],
    pub dynamic_rendering: bool,
    pub acceleration_structure: bool,
    pub buffer_device_address: bool,
}

/// A physical device represents a Vulkan-capable GPU or other hardware that can execute Vulkan commands.
pub struct PhysicalDevice {
    #[allow(dead_code)]
    instance: Arc<RawInstance>,
    physical: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    families: Vec<vk::QueueFamilyProperties>,
    extensions: Vec<DeviceExtensionProperties>,
    memory_types: Vec<MemoryType>,
    memory_heaps: Vec<MemoryHeap>,
}

impl PhysicalDevice {
    /// Get the properties of this physical device.
    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    /// Get the properties of queue families available on this physical device.
    pub fn queue_families(&self) -> &[vk::QueueFamilyProperties] {
        &self.families
    }

    /// Get the number of extensions supported by this physical device.
    pub fn extensions(&self) -> &[DeviceExtensionProperties] {
        &self.extensions
    }

    /// Get the name of this physical device.
    pub fn name(&self) -> Cow<'_, str> {
        self.properties
            .device_name_as_c_str()
            .unwrap_or_default()
            .to_string_lossy()
    }

    /// Get the memory types available on this physical device.
    pub fn memory_types(&self) -> &[MemoryType] {
        &self.memory_types
    }

    /// Get the memory heaps available on this physical device.
    pub fn memory_heaps(&self) -> &[MemoryHeap] {
        &self.memory_heaps
    }

    /// Check if this physical device is a discrete GPU.
    pub fn is_discrete(&self) -> bool {
        self.properties().device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    }

    pub fn is_integrated(&self) -> bool {
        self.properties().device_type == vk::PhysicalDeviceType::INTEGRATED_GPU
    }

    pub fn is_virtual(&self) -> bool {
        self.properties().device_type == vk::PhysicalDeviceType::VIRTUAL_GPU
    }

    pub fn raw_physical_device(&self) -> vk::PhysicalDevice {
        self.physical
    }

    #[track_caller]
    pub fn create_device(&self, desc: &DeviceDescriptor<'_>) -> (Device, Vec<Vec<Queue>>) {
        self.try_create_device(desc)
            .expect("Failed to create device")
    }

    pub fn try_create_device(
        &self,
        desc: &DeviceDescriptor<'_>,
    ) -> Result<(Device, Vec<Vec<Queue>>), vk::Result> {
        let mut queue_create_infos = Vec::with_capacity(desc.queues.len());

        for queue in desc.queues {
            let create_info = vk::DeviceQueueCreateInfo {
                queue_family_index: queue.family,
                queue_count: queue.priorities.len() as u32,
                p_queue_priorities: queue.priorities.as_ptr(),
                ..Default::default()
            };

            queue_create_infos.push(create_info);
        }

        let extensions: Vec<_> = desc.extensions.iter().map(|ext| ext.as_ptr()).collect();

        let mut dynamic_rendering = vk::PhysicalDeviceDynamicRenderingFeatures {
            dynamic_rendering: desc.dynamic_rendering.into(),
            ..Default::default()
        };

        let mut acceleration_structure = vk::PhysicalDeviceAccelerationStructureFeaturesKHR {
            acceleration_structure: desc.acceleration_structure.into(),
            ..Default::default()
        };

        let mut buffer_device_address = vk::PhysicalDeviceBufferDeviceAddressFeatures {
            buffer_device_address: desc.buffer_device_address.into(),
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        }
        .push_next(&mut dynamic_rendering)
        .push_next(&mut acceleration_structure)
        .push_next(&mut buffer_device_address);

        let raw_device = unsafe {
            self.instance
                .instance
                .create_device(self.physical, &create_info, None)?
        };

        let device = Device {
            raw: Arc::new(RawDevice {
                instance: self.instance.clone(),
                device: raw_device,
                physical: Self {
                    instance: self.instance.clone(),
                    physical: self.physical,
                    properties: self.properties,
                    families: self.families.clone(),
                    extensions: self.extensions.clone(),
                    memory_types: self.memory_types.clone(),
                    memory_heaps: self.memory_heaps.clone(),
                },
            }),
        };

        let mut families = Vec::with_capacity(desc.queues.len());

        for queue in desc.queues {
            let mut queues = Vec::with_capacity(queue.priorities.len());

            for queue_index in 0..queue.priorities.len() as u32 {
                let handle = unsafe {
                    device
                        .raw_device()
                        .get_device_queue(queue.family, queue_index)
                };

                queues.push(Queue {
                    device: device.raw.clone(),
                    queue: handle,
                    family: queue.family,
                    index: queue_index,
                });
            }

            families.push(queues);
        }

        Ok((device, families))
    }
}

impl Instance {
    #[track_caller]
    pub fn physical_devices(&self) -> Vec<PhysicalDevice> {
        self.try_physical_devices()
            .expect("Failed to enumerate physical devices")
    }

    pub fn try_physical_devices(&self) -> Result<Vec<PhysicalDevice>, vk::Result> {
        let devices = unsafe { self.raw_instance().enumerate_physical_devices()? };
        let devices = devices
            .into_iter()
            .map(|physical| unsafe {
                let properties = self.raw_instance().get_physical_device_properties(physical);

                let families = self
                    .raw_instance()
                    .get_physical_device_queue_family_properties(physical);

                let extensions = self
                    .raw_instance()
                    .enumerate_device_extension_properties(physical)
                    .into_iter()
                    .flatten()
                    .map(|ext| DeviceExtensionProperties {
                        name: DeviceExtension(ext.extension_name),
                        version: Version::from_raw(ext.spec_version),
                    })
                    .collect();

                let memory_properties = self
                    .raw_instance()
                    .get_physical_device_memory_properties(physical);

                let mut memory_types =
                    Vec::with_capacity(memory_properties.memory_type_count as usize);
                let mut memory_heaps =
                    Vec::with_capacity(memory_properties.memory_heap_count as usize);

                for memory_type in memory_properties.memory_types_as_slice() {
                    memory_types.push(MemoryType {
                        properties: MemoryProperties::from_bits(
                            memory_type.property_flags.as_raw(),
                        )
                        .expect("Invalid memory properties"),
                        heap_index: memory_type.heap_index,
                    });
                }

                for memory_heap in memory_properties.memory_heaps_as_slice() {
                    memory_heaps.push(MemoryHeap {
                        size: memory_heap.size,
                        device_local: memory_heap
                            .flags
                            .contains(vk::MemoryHeapFlags::DEVICE_LOCAL),
                    });
                }

                PhysicalDevice {
                    instance: self.raw.clone(),
                    physical,
                    properties,
                    families,
                    extensions,
                    memory_types,
                    memory_heaps,
                }
            })
            .collect();

        Ok(devices)
    }
}

pub struct Device {
    pub(crate) raw: Arc<RawDevice>,
}

impl Device {
    pub fn raw_instance(&self) -> &ash::Instance {
        &self.raw.instance.instance
    }

    pub fn raw_device(&self) -> &ash::Device {
        &self.raw.device
    }

    pub fn physical(&self) -> &PhysicalDevice {
        &self.raw.physical
    }

    #[track_caller]
    pub fn wait_idle(&self) {
        self.try_wait_idle()
            .expect("Failed to wait for device to become idle");
    }

    pub fn try_wait_idle(&self) -> Result<(), vk::Result> {
        unsafe { self.raw.device.device_wait_idle() }
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device").finish()
    }
}

pub(crate) struct RawDevice {
    #[allow(dead_code)]
    pub(crate) physical: PhysicalDevice,
    pub(crate) instance: Arc<RawInstance>,
    pub(crate) device: ash::Device,
}

impl Drop for RawDevice {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.device.handle(), "Destroying device");
            self.device.destroy_device(None);
        }
    }
}

impl Deref for RawDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
