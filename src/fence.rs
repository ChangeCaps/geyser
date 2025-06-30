use std::{fmt, sync::Arc};

use ash::vk;

use crate::{Device, RawDevice};

impl Device {
    pub fn create_fence(&self, signalled: bool) -> Fence {
        self.try_create_fence(signalled)
            .expect("Failed to create fence")
    }

    pub fn try_create_fence(&self, signalled: bool) -> Result<Fence, vk::Result> {
        let create_info = vk::FenceCreateInfo {
            flags: if signalled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            },
            ..Default::default()
        };

        let fence = unsafe { self.raw_device().create_fence(&create_info, None) }?;

        let raw_fence = RawFence {
            device: self.raw.clone(),
            fence,
        };

        Ok(Fence {
            raw: Arc::new(raw_fence),
        })
    }

    #[track_caller]
    pub fn wait_for_fences(&self, fences: &[&Fence], wait_all: bool, timeout: Option<u64>) {
        self.try_wait_for_fences(fences, wait_all, timeout)
            .expect("Failed to wait for fences")
    }

    pub fn try_wait_for_fences(
        &self,
        fences: &[&Fence],
        wait_all: bool,
        timeout: Option<u64>,
    ) -> Result<(), vk::Result> {
        let raw_fences: Vec<vk::Fence> = fences.iter().map(|f| f.raw_fence()).collect();
        let timeout = timeout.unwrap_or(vk::WHOLE_SIZE);

        unsafe {
            self.raw_device()
                .wait_for_fences(&raw_fences, wait_all, timeout)?;
        }

        Ok(())
    }

    #[track_caller]
    pub fn reset_fences(&self, fences: &[&Fence]) {
        self.try_reset_fences(fences)
            .expect("Failed to reset fences")
    }

    pub fn try_reset_fences(&self, fences: &[&Fence]) -> Result<(), vk::Result> {
        let raw_fences: Vec<vk::Fence> = fences.iter().map(|f| f.raw_fence()).collect();

        unsafe {
            self.raw_device().reset_fences(&raw_fences)?;
        }

        Ok(())
    }
}

pub struct Fence {
    pub(crate) raw: Arc<RawFence>,
}

impl Fence {
    pub fn raw_fence(&self) -> vk::Fence {
        self.raw.fence
    }

    #[track_caller]
    pub fn wait(&self, timeout: Option<u64>) {
        self.try_wait(timeout).expect("Failed to wait for fence")
    }

    pub fn try_wait(&self, timeout: Option<u64>) -> Result<(), vk::Result> {
        let timeout = timeout.unwrap_or(vk::WHOLE_SIZE);

        unsafe {
            (self.raw.device.device).wait_for_fences(&[self.raw.fence], true, timeout)?;
        }

        Ok(())
    }

    #[track_caller]
    pub fn reset(&self) {
        self.try_reset().expect("Failed to reset fence")
    }

    pub fn try_reset(&self) -> Result<(), vk::Result> {
        unsafe {
            self.raw.device.device.reset_fences(&[self.raw.fence])?;
        }

        Ok(())
    }
}

impl fmt::Debug for Fence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fence").finish()
    }
}

pub(crate) struct RawFence {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) fence: vk::Fence,
}

impl Drop for RawFence {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            tracing::trace!(
                handle = ?self.fence,
                "Destroying fence",
            );
            self.device.device.destroy_fence(self.fence, None);
        }
    }
}
