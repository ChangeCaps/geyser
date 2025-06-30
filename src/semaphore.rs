use std::{fmt, sync::Arc};

use ash::vk;

use crate::{Device, RawDevice};

impl Device {
    #[track_caller]
    pub fn create_semaphore(&self) -> Semaphore {
        self.try_create_semaphore()
            .expect("Failed to create semaphore")
    }

    pub fn try_create_semaphore(&self) -> Result<Semaphore, vk::Result> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe { self.raw_device().create_semaphore(&semaphore_info, None)? };

        let raw = RawSemaphore {
            device: self.raw.clone(),
            semaphore,
        };

        Ok(Semaphore { raw: Arc::new(raw) })
    }
}

pub struct Semaphore {
    pub(crate) raw: Arc<RawSemaphore>,
}

impl Semaphore {
    pub fn raw_semaphore(&self) -> vk::Semaphore {
        self.raw.semaphore
    }
}

impl fmt::Debug for Semaphore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Semaphore").finish()
    }
}

pub(crate) struct RawSemaphore {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) semaphore: vk::Semaphore,
}

impl Drop for RawSemaphore {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            tracing::trace!(
                handle = ?self.semaphore,
                "Destroying semaphore",
            );
            self.device.destroy_semaphore(self.semaphore, None);
        }
    }
}
