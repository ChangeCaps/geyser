use std::{fmt, sync::Arc};

use ash::vk;

use crate::{
    Access, Blas, Buffer, CommandBufferLevel, CommandBufferUsages, CommandPoolFlags, Dependencies,
    Image, ImageAspects, ImageLayout, ImageView, PipelineStages, Queue, RawAccel, RawBuffer,
    RawDevice, RawImage, RawImageView, Tlas,
};

impl Queue {
    #[track_caller]
    pub fn create_command_pool(&self, flags: CommandPoolFlags) -> CommandPool {
        self.try_create_command_pool(flags)
            .expect("Failed to create command pool")
    }

    pub fn try_create_command_pool(
        &self,
        flags: CommandPoolFlags,
    ) -> Result<CommandPool, vk::Result> {
        let create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::from_raw(flags.bits()),
            queue_family_index: self.family(),
            ..Default::default()
        };

        let command_pool = unsafe { self.device.create_command_pool(&create_info, None)? };

        let raw = Arc::new(RawCommandPool {
            device: self.device.clone(),
            command_pool,
            flags,
            queue_family_index: self.family(),
        });

        Ok(CommandPool { raw })
    }
}

pub struct CommandPool {
    pub(crate) raw: Arc<RawCommandPool>,
}

impl CommandPool {
    pub fn flags(&self) -> CommandPoolFlags {
        self.raw.flags
    }

    pub fn queue_family_index(&self) -> u32 {
        self.raw.queue_family_index
    }

    #[track_caller]
    pub fn allocate_command_buffer(&self, level: CommandBufferLevel) -> CommandBuffer {
        self.try_allocate_command_buffer(level)
            .expect("Failed to allocate command buffer")
    }

    pub fn try_allocate_command_buffer(
        &self,
        level: CommandBufferLevel,
    ) -> Result<CommandBuffer, vk::Result> {
        Ok(self.try_allocate_command_buffers(1, level)?.remove(0))
    }

    #[track_caller]
    pub fn allocate_command_buffers(
        &self,
        count: u32,
        level: CommandBufferLevel,
    ) -> Vec<CommandBuffer> {
        self.try_allocate_command_buffers(count, level)
            .expect("Failed to allocate command buffers")
    }

    pub fn try_allocate_command_buffers(
        &self,
        count: u32,
        level: CommandBufferLevel,
    ) -> Result<Vec<CommandBuffer>, vk::Result> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: self.raw.command_pool,
            level: vk::CommandBufferLevel::from_raw(level.as_raw()),
            command_buffer_count: count,
            ..Default::default()
        };

        let buffers = unsafe { self.raw.device.allocate_command_buffers(&allocate_info)? };

        let buffers = buffers
            .into_iter()
            .map(|buffer| CommandBuffer {
                raw: Arc::new(RawCommandBuffer {
                    command_pool: self.raw.clone(),
                    command_buffer: buffer,
                    level,
                }),
                images: Vec::new(),
                accels: Vec::new(),
                buffers: Vec::new(),
                image_views: Vec::new(),
            })
            .collect();

        Ok(buffers)
    }
}

impl fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandPool")
            .field("command_pool", &self.raw.command_pool)
            .field("flags", &self.raw.flags)
            .field("queue_family_index", &self.raw.queue_family_index)
            .finish()
    }
}

pub(crate) struct RawCommandPool {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) flags: CommandPoolFlags,
    pub(crate) queue_family_index: u32,
}

impl Drop for RawCommandPool {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(
                handle = ?self.command_pool,
                "Destroying command pool",
            );

            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}

pub struct CommandBuffer {
    pub(crate) raw: Arc<RawCommandBuffer>,

    // keep track of resources used in the command buffer
    // to ensure they are not dropped while the command buffer is still in use
    pub(crate) images: Vec<Arc<RawImage>>,
    pub(crate) accels: Vec<Arc<RawAccel>>,
    pub(crate) buffers: Vec<Arc<RawBuffer>>,
    pub(crate) image_views: Vec<Arc<RawImageView>>,
}

pub(crate) struct RawCommandBuffer {
    pub(crate) command_pool: Arc<RawCommandPool>,
    pub(crate) command_buffer: vk::CommandBuffer,
    pub(crate) level: CommandBufferLevel,
}

impl RawCommandBuffer {
    fn device(&self) -> &ash::Device {
        &self.command_pool.device
    }
}

impl CommandBuffer {
    pub(crate) fn track_image(&mut self, image: &Image) {
        self.images.push(image.raw.clone());
    }

    pub(crate) fn track_blas(&mut self, blas: &Blas) {
        self.accels.push(blas.raw.clone());
    }

    pub(crate) fn track_tlas(&mut self, tlas: &Tlas) {
        self.accels.push(tlas.raw.clone());
    }

    pub(crate) fn track_image_view(&mut self, image_view: &ImageView) {
        self.image_views.push(image_view.raw.clone());
    }

    pub(crate) fn track_buffer(&mut self, buffer: &Buffer) {
        self.buffers.push(buffer.raw.clone());
    }

    pub(crate) fn clear_tracked_resources(&mut self) {
        self.images.clear();
        self.accels.clear();
        self.buffers.clear();
        self.image_views.clear();
    }

    pub fn raw_command_buffer(&self) -> vk::CommandBuffer {
        self.raw.command_buffer
    }

    pub fn level(&self) -> CommandBufferLevel {
        self.raw.level
    }

    #[track_caller]
    pub fn begin(&mut self, usages: CommandBufferUsages) -> CommandEncoder<'_> {
        self.try_begin(usages)
            .expect("Failed to begin command buffer")
    }

    pub fn try_begin(
        &mut self,
        usages: CommandBufferUsages,
    ) -> Result<CommandEncoder<'_>, vk::Result> {
        assert!(
            self.raw.level == CommandBufferLevel::Primary,
            "Command buffer must be primary to record commands"
        );

        assert!(
            !(usages.contains(CommandBufferUsages::ONE_TIME_SUBMIT)
                && usages.contains(CommandBufferUsages::SIMULTANEOUS_USE)),
            "Command buffer cannot be recorded with both ONE_TIME_SUBMIT and SIMULTANEOUS_USE usages"
        );

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::from_raw(usages.bits()),
            ..Default::default()
        };

        // clear the tracked resources
        self.clear_tracked_resources();

        unsafe {
            self.raw
                .device()
                .begin_command_buffer(self.raw.command_buffer, &begin_info)?;
        }

        Ok(CommandEncoder {
            command_buffer: self,
            finished: false,
        })
    }
}

impl fmt::Debug for CommandBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandBuffer")
            .field("command_buffer", &self.raw.command_buffer)
            .field("level", &self.raw.level)
            .finish()
    }
}

pub struct CommandEncoder<'a> {
    pub(crate) command_buffer: &'a mut CommandBuffer,
    pub(crate) finished: bool,
}

impl<'a> CommandEncoder<'a> {
    pub(crate) fn device(&self) -> &ash::Device {
        self.command_buffer.raw.device()
    }

    #[track_caller]
    pub fn end(self) {
        self.try_end().expect("Failed to finish command buffer");
    }

    pub fn try_end(mut self) -> Result<(), vk::Result> {
        unsafe {
            self.device()
                .end_command_buffer(self.command_buffer.raw.command_buffer)?;
        }

        self.finished = true;

        Ok(())
    }

    pub fn pipeline_barrier(&mut self, barrier: &PipelineBarrier<'_>) {
        assert!(
            !barrier.memory_barriers.is_empty() || !barrier.image_barriers.is_empty(),
            "At least one memory or image barrier must be provided"
        );

        let memory_barriers: Vec<vk::MemoryBarrier> = barrier
            .memory_barriers
            .iter()
            .map(|barrier| vk::MemoryBarrier {
                src_access_mask: vk::AccessFlags::from_raw(barrier.src_access.bits()),
                dst_access_mask: vk::AccessFlags::from_raw(barrier.dst_access.bits()),
                ..Default::default()
            })
            .collect();

        let image_barriers: Vec<vk::ImageMemoryBarrier> = barrier
            .image_barriers
            .iter()
            .map(|barrier| {
                self.command_buffer.track_image(barrier.image);

                vk::ImageMemoryBarrier {
                    src_access_mask: vk::AccessFlags::from_raw(barrier.src_access.bits()),
                    dst_access_mask: vk::AccessFlags::from_raw(barrier.dst_access.bits()),
                    old_layout: vk::ImageLayout::from_raw(barrier.old_layout.as_raw()),
                    new_layout: vk::ImageLayout::from_raw(barrier.new_layout.as_raw()),
                    src_queue_family_index: barrier.src_family.unwrap_or(vk::QUEUE_FAMILY_IGNORED),
                    dst_queue_family_index: barrier.dst_family.unwrap_or(vk::QUEUE_FAMILY_IGNORED),
                    image: barrier.image.raw_image(),
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::from_raw(barrier.aspects.bits()),
                        base_mip_level: barrier.base_mip_level,
                        level_count: barrier.level_count,
                        base_array_layer: barrier.base_array_layer,
                        layer_count: barrier.layer_count,
                    },
                    ..Default::default()
                }
            })
            .collect();

        unsafe {
            self.device().cmd_pipeline_barrier(
                self.command_buffer.raw.command_buffer,
                vk::PipelineStageFlags::from_raw(barrier.src_stages.bits()),
                vk::PipelineStageFlags::from_raw(barrier.dst_stages.bits()),
                vk::DependencyFlags::from_raw(barrier.dependencies.bits()),
                &memory_barriers,
                &[],
                &image_barriers,
            );
        }
    }
}

impl fmt::Debug for CommandEncoder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandEncoder")
            .field("command_buffer", &self.command_buffer)
            .finish()
    }
}

impl Drop for CommandEncoder<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }

        tracing::warn!("CommandEncoder dropped without being finished.");

        unsafe {
            self.device()
                .end_command_buffer(self.command_buffer.raw.command_buffer)
                .expect("Failed to end command buffer");
        }
    }
}

#[derive(Clone, Debug)]
pub struct PipelineBarrier<'a> {
    pub src_stages: PipelineStages,
    pub dst_stages: PipelineStages,
    pub dependencies: Dependencies,
    pub memory_barriers: &'a [MemoryBarrier],
    pub image_barriers: &'a [ImageBarrier<'a>],
}

impl Default for PipelineBarrier<'_> {
    fn default() -> Self {
        Self {
            src_stages: PipelineStages::BOTTOM_OF_PIPE,
            dst_stages: PipelineStages::TOP_OF_PIPE,
            dependencies: Dependencies::empty(),
            memory_barriers: &[],
            image_barriers: &[],
        }
    }
}

#[derive(Clone, Debug)]
pub struct MemoryBarrier {
    pub src_access: Access,
    pub dst_access: Access,
}

#[derive(Clone, Debug)]
pub struct ImageBarrier<'a> {
    pub src_access: Access,
    pub dst_access: Access,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_family: Option<u32>,
    pub dst_family: Option<u32>,
    pub image: &'a Image,
    pub aspects: ImageAspects,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl<'a> ImageBarrier<'a> {
    pub fn default(image: &'a Image) -> Self {
        Self {
            src_access: Access::empty(),
            dst_access: Access::empty(),
            old_layout: ImageLayout::Undefined,
            new_layout: ImageLayout::Undefined,
            src_family: None,
            dst_family: None,
            image,
            aspects: ImageAspects::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}
