use std::{fmt, sync::Arc};

use ash::{khr, vk};

use crate::{CommandBuffer, Fence, PipelineStages, RawDevice, Semaphore, SwapchainImage};

pub struct Queue {
    #[allow(dead_code)]
    pub(crate) device: Arc<RawDevice>,
    pub(crate) queue: vk::Queue,
    pub(crate) family: u32,
    pub(crate) index: u32,
}

impl Queue {
    pub fn as_raw(&self) -> vk::Queue {
        self.queue
    }

    /// Get the index of the family this queue belongs to.
    pub fn family(&self) -> u32 {
        self.family
    }

    /// Get the index of this queue within its family.
    pub fn index(&self) -> u32 {
        self.index
    }

    #[track_caller]
    pub fn submit(&self, submits: &[Submit<'_>], fence: Option<&Fence>) {
        self.try_submit(submits, fence)
            .expect("Failed to submit queue");
    }

    pub fn try_submit(
        &self,
        submits: &[Submit<'_>],
        fence: Option<&Fence>,
    ) -> Result<(), vk::Result> {
        assert!(
            !submits.is_empty(),
            "No submits provided for queue submission"
        );

        let mut raw_submits = Vec::with_capacity(submits.len());
        let mut resources = Vec::with_capacity(submits.len());

        for submit in submits {
            let wait_semaphores: Vec<_> = submit
                .wait_semaphores
                .iter()
                .map(|s| s.semaphore.raw_semaphore())
                .collect();

            let wait_dst_stage_mask: Vec<_> = submit
                .wait_semaphores
                .iter()
                .map(|s| vk::PipelineStageFlags::from_raw(s.dst_stage_mask.bits()))
                .collect();

            let signal_semaphores: Vec<_> = submit
                .signal_semaphores
                .iter()
                .map(|s| s.raw_semaphore())
                .collect();

            let command_buffers: Vec<_> = submit
                .command_buffers
                .iter()
                .map(|c| c.raw_command_buffer())
                .collect();

            raw_submits.push(vk::SubmitInfo {
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_dst_stage_mask.as_ptr(),
                command_buffer_count: command_buffers.len() as u32,
                p_command_buffers: command_buffers.as_ptr(),
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
                ..Default::default()
            });

            // store resource vectors so they aren't dropped too early
            resources.push((
                wait_semaphores,
                wait_dst_stage_mask,
                command_buffers,
                signal_semaphores,
            ));
        }

        unsafe {
            self.device.device.queue_submit(
                self.queue,
                &raw_submits,
                fence.map_or(vk::Fence::null(), Fence::raw_fence),
            )
        }
    }

    pub fn present(
        &self,
        images: &[SwapchainImage<'_>],
        wait_semaphores: &[&Semaphore],
    ) -> Result<bool, vk::Result> {
        assert!(!images.is_empty(), "No images provided for presentation",);

        let wait_semaphores: Vec<_> = wait_semaphores.iter().map(|s| s.raw_semaphore()).collect();
        let swapchains: Vec<_> = images.iter().map(|s| s.swapchain).collect();
        let image_indices: Vec<_> = images.iter().map(|s| s.index).collect();

        assert_eq!(swapchains.len(), image_indices.len());

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: image_indices.as_ptr(),
            ..Default::default()
        };

        unsafe {
            let khr = khr::swapchain::Device::new(
                &self.device.instance.instance,
                &self.device.device, //
            );

            khr.queue_present(self.queue, &present_info)
        }
    }
}

impl fmt::Debug for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue").finish()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Submit<'a> {
    pub wait_semaphores: &'a [WaitSemaphore<'a>],
    pub command_buffers: &'a [&'a CommandBuffer],
    pub signal_semaphores: &'a [&'a Semaphore],
}

#[derive(Clone, Debug)]
pub struct WaitSemaphore<'a> {
    pub semaphore: &'a Semaphore,
    pub dst_stage_mask: PipelineStages,
}
