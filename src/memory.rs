use std::{
    fmt,
    ops::{Deref, DerefMut, Range},
    slice,
    sync::Arc,
};

use ash::vk;

use crate::{Device, MemoryAllocateFlags, MemoryProperties, PhysicalDevice, RawDevice};

impl Device {
    #[track_caller]
    pub fn allocate_memory(
        &self,
        allocation_size: u64,
        memory_type_index: u32,
        allocate_flags: MemoryAllocateFlags,
    ) -> Memory {
        self.try_allocate_memory(allocation_size, memory_type_index, allocate_flags)
            .expect("Failed to allocate memory")
    }

    pub fn try_allocate_memory(
        &self,
        allocation_size: u64,
        memory_type_index: u32,
        allocate_flags: MemoryAllocateFlags,
    ) -> Result<Memory, vk::Result> {
        let mut memory_allocate_flags = vk::MemoryAllocateFlagsInfo {
            flags: vk::MemoryAllocateFlags::from_raw(allocate_flags.bits()),
            ..Default::default()
        };

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size,
            memory_type_index,
            ..Default::default()
        }
        .push_next(&mut memory_allocate_flags);

        let memory_handle = unsafe { self.raw.allocate_memory(&allocate_info, None)? };

        let raw = RawMemory {
            device: self.raw.clone(),
            memory: memory_handle,
            size: allocation_size,
        };

        Ok(Memory { raw: Arc::new(raw) })
    }
}

impl PhysicalDevice {
    pub fn available_memory_types(
        &self,
        requirements: MemoryRequirements,
        properties: MemoryProperties,
    ) -> Vec<u32> {
        let mut types = Vec::new();

        for (i, memory_type) in self.memory_types().iter().enumerate() {
            let mask = 1 << i;
            if requirements.memory_type_bits & mask != 0
                && memory_type.properties.contains(properties)
            {
                types.push(i as u32);
            }
        }

        types
    }
}

pub struct Memory {
    pub(crate) raw: Arc<RawMemory>,
}

impl Memory {
    pub fn size(&self) -> u64 {
        self.raw.size
    }

    pub fn map(&mut self, range: Range<u64>) -> MappedMemory<'_> {
        self.try_map(range).expect("Failed to map memory")
    }

    pub fn try_map(&mut self, range: Range<u64>) -> Result<MappedMemory<'_>, vk::Result> {
        assert!(
            range.start <= self.size() && range.end <= self.size(),
            "Memory range out of bounds: {:?} for memory size {}",
            range,
            self.size()
        );

        assert!(
            range.start < range.end,
            "Memory range must not be empty: {:?}",
            range
        );

        let size = range.end - range.start;

        let data = unsafe {
            self.raw.device.map_memory(
                self.raw.memory,
                range.start,
                size,
                vk::MemoryMapFlags::empty(),
            )?
        };

        let data = unsafe { slice::from_raw_parts_mut(data as *mut u8, size as usize) };

        Ok(MappedMemory { data, memory: self })
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("memory", &self.raw.memory)
            .finish()
    }
}

pub(crate) struct RawMemory {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) memory: vk::DeviceMemory,
    pub(crate) size: u64,
}

impl Drop for RawMemory {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.memory, "Freeing memory");
            self.device.free_memory(self.memory, None);
        }
    }
}

#[derive(Debug)]
pub struct MappedMemory<'a> {
    pub data: &'a mut [u8],
    memory: &'a Memory,
}

impl MappedMemory<'_> {
    pub fn unmap(self) {
        // unmapping is handled in the Drop implementation
        // of MappedMemory, so this method is a no-op.
    }
}

impl Deref for MappedMemory<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl DerefMut for MappedMemory<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl Drop for MappedMemory<'_> {
    fn drop(&mut self) {
        unsafe {
            self.memory.raw.device.unmap_memory(self.memory.raw.memory);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemoryRequirements {
    pub size: u64,
    pub alignment: u64,
    pub memory_type_bits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemoryType {
    pub properties: MemoryProperties,
    pub heap_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemoryHeap {
    pub size: u64,
    pub device_local: bool,
}
