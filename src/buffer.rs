use std::{
    fmt,
    sync::{Arc, Mutex},
};

use ash::vk;

use crate::{
    BufferFlags, BufferUsages, Device, Memory, MemoryAllocateFlags, MemoryProperties,
    MemoryRequirements, RawDevice, RawMemory, SharingMode,
};

impl Device {
    #[track_caller]
    pub fn create_buffer(&self, descriptor: &BufferDescriptor) -> Buffer {
        self.try_create_buffer(descriptor)
            .expect("Failed to create buffer")
    }

    pub fn try_create_buffer(&self, descriptor: &BufferDescriptor) -> Result<Buffer, vk::Result> {
        assert!(descriptor.size > 0, "Buffer size must be greater than zero");
        assert!(
            !descriptor.usages.is_empty(),
            "Buffer usages must not be empty",
        );

        let create_info = vk::BufferCreateInfo {
            flags: vk::BufferCreateFlags::from_raw(descriptor.flags.bits()),
            size: descriptor.size,
            usage: vk::BufferUsageFlags::from_raw(descriptor.usages.bits()),
            sharing_mode: vk::SharingMode::from_raw(descriptor.sharing_mode.as_raw()),
            queue_family_index_count: descriptor.families.len() as u32,
            p_queue_family_indices: descriptor.families.as_ptr(),
            ..Default::default()
        };

        let buffer = unsafe { self.raw.device.create_buffer(&create_info, None)? };

        let raw_buffer = RawBuffer {
            device: self.raw.clone(),
            memory: Mutex::new(None),
            handle: buffer,

            flags: descriptor.flags,
            size: descriptor.size,
            usages: descriptor.usages,
            sharing_mode: descriptor.sharing_mode,
            families: descriptor.families.clone(),
        };

        Ok(Buffer {
            raw: Arc::new(raw_buffer),
        })
    }

    #[track_caller]
    pub fn allocate_buffer_memory(
        &self,
        buffer: &Buffer,
        properties: MemoryProperties,
        allocate_flags: MemoryAllocateFlags,
    ) -> Memory {
        self.try_allocate_buffer_memory(buffer, properties, allocate_flags)
            .expect("Failed to allocate buffer memory")
    }

    pub fn try_allocate_buffer_memory(
        &self,
        buffer: &Buffer,
        properties: MemoryProperties,
        allocate_flags: MemoryAllocateFlags,
    ) -> Result<Memory, vk::Result> {
        let memory_types = self
            .physical()
            .available_memory_types(buffer.memory_requirements(), properties);

        if memory_types.is_empty() {
            return Err(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY);
        }

        let memory_type_index = memory_types[0];

        let memory = self.try_allocate_memory(
            buffer.memory_requirements().size, //
            memory_type_index,
            allocate_flags,
        )?;

        buffer.try_bind_memory(&memory, 0)?;

        Ok(memory)
    }
}

#[derive(Clone, Debug)]
pub struct BufferDescriptor {
    pub flags: BufferFlags,
    pub size: u64,
    pub usages: BufferUsages,
    pub sharing_mode: SharingMode,
    pub families: Vec<u32>,
}

impl Default for BufferDescriptor {
    fn default() -> Self {
        Self {
            flags: BufferFlags::empty(),
            size: 0,
            usages: BufferUsages::empty(),
            sharing_mode: SharingMode::Exclusive,
            families: Vec::new(),
        }
    }
}

pub struct Buffer {
    pub(crate) raw: Arc<RawBuffer>,
}

impl Buffer {
    pub fn flags(&self) -> BufferFlags {
        self.raw.flags
    }

    pub fn size(&self) -> u64 {
        self.raw.size
    }

    pub fn usages(&self) -> BufferUsages {
        self.raw.usages
    }

    pub fn sharing_mode(&self) -> SharingMode {
        self.raw.sharing_mode
    }

    pub fn families(&self) -> &[u32] {
        &self.raw.families
    }

    pub fn memory_requirements(&self) -> MemoryRequirements {
        let requirements =
            unsafe { (self.raw.device).get_buffer_memory_requirements(self.raw.handle) };

        MemoryRequirements {
            size: requirements.size,
            alignment: requirements.alignment,
            memory_type_bits: requirements.memory_type_bits,
        }
    }

    pub fn device_address(&self) -> u64 {
        let address_info = vk::BufferDeviceAddressInfo {
            buffer: self.raw.handle,
            ..Default::default()
        };

        unsafe { (self.raw.device).get_buffer_device_address(&address_info) }
    }

    pub fn bind_memory(&self, memory: &Memory, offset: u64) {
        self.try_bind_memory(memory, offset)
            .expect("Failed to bind buffer memory");
    }

    pub fn try_bind_memory(&self, memory: &Memory, offset: u64) -> Result<(), vk::Result> {
        unsafe {
            (self.raw.device).bind_buffer_memory(self.raw.handle, memory.raw.memory, offset)?;
        }

        if let Ok(mut raw_memory) = self.raw.memory.lock() {
            *raw_memory = Some(memory.raw.clone());
        }

        Ok(())
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
            .field("raw", &self.raw.handle)
            .finish()
    }
}

pub(crate) struct RawBuffer {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) memory: Mutex<Option<Arc<RawMemory>>>,
    pub(crate) handle: vk::Buffer,

    pub(crate) flags: BufferFlags,
    pub(crate) size: u64,
    pub(crate) usages: BufferUsages,
    pub(crate) sharing_mode: SharingMode,
    pub(crate) families: Vec<u32>,
}

impl Drop for RawBuffer {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(
                handle = ?self.handle,
                "Destroying buffer",
            );

            self.device.device.destroy_buffer(self.handle, None);
        }
    }
}
