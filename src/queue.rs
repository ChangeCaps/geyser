use std::{fmt, sync::Arc};

use ash::vk;

use crate::DeviceInner;

pub struct Queue {
    pub(crate) inner: Arc<QueueInner>,
}

impl Queue {
    pub fn family_index(&self) -> u32 {
        self.inner.family_index
    }

    pub fn queue_index(&self) -> u32 {
        self.inner.queue_index
    }
}

impl fmt::Debug for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.inner.handle)
            .field("family_index", &self.inner.family_index)
            .field("queue_index", &self.inner.queue_index)
            .finish()
    }
}

pub(crate) struct QueueInner {
    pub(crate) handle: vk::Queue,

    pub(crate) device: Arc<DeviceInner>,

    pub(crate) family_index: u32,
    pub(crate) queue_index: u32,
}
