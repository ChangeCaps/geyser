//! This module contains a struct for instancing [`vulkano`]
//! and provides methods for easily creating buffers ect.

use vulkano::{
    device,
    instance,
    buffer::*,
};
use std::sync::Arc;

/// This is a struct that holds an [`Arc`]<[`Instance`]>, [`Arc`]<[`Device`](device::Device)> and an [`Arc`]<[`Queue`](device::Device)>.
/// This serves the purpose of making it easier to create everything needed for your GPU calculations.
/// Note that you should try to **never** call [`Instance::new`] more than once!
/// 
/// Here we initialize vulkan and we create a [`CpuAccessibleBuffer`] containing 69 42s.
/// # Example
/// ```
/// # extern crate geyser;
/// use geyser::instance::Instance;
/// 
/// let inst = Instance::new();
/// 
/// let buf = inst.create_buffer_from_data(vec![42; 69]);
/// ```
#[allow(dead_code)]
pub struct Instance {
    instance: Arc<instance::Instance>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
}

#[allow(dead_code)]
impl Instance {
    /// Creates a new [`Instance`].
    /// 
    /// It uses the first [`QueueFamily`](instance::QueueFamily) that supports graphics and the first [`Queue`](device::Queue) in that [`QueueFamily`](instance::QueueFamily)
    pub fn new() -> Instance {
        let instance = instance::Instance::new(None, &instance::InstanceExtensions::none(), None).expect("Failed to create instance");

        let physical = instance::PhysicalDevice::enumerate(&instance).next().expect("Fail to create physical instance");

        let queue_family = physical.queue_families()
            .find(|q| q.supports_graphics())
            .expect("No queue families support graphics");

        let (device, mut queues) = device::Device::new(physical, &device::Features::none(), &device::DeviceExtensions::none(),
                                               [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");

        let queue = queues.next().unwrap();


        Instance {
            instance,
            device,
            queue,
        }
    }

    //Interfaces

    /// Returns a clone of the [`Arc`]<[`Instance`]>
    /// inside the [`Instance`]
    pub fn get_instance(&self) -> Arc<instance::Instance> {
        self.instance.clone()
    }

    /// Returns a clone of the [`Arc`]<[`Device`](device::Device)>
    /// inside the [`Instance`]
    pub fn get_device(&self) -> Arc<device::Device> {
        self.device.clone()
    }

    /// Returns a clone of the [`Arc`]<[`Queue`](device::Device)>
    /// inside the [`Instance`]
    pub fn get_queue(&self) -> Arc<device::Queue> {
        self.queue.clone()
    }

    //Stuff

    /// Creates an [`Arc`]<[`CpuAccessibleBuffer`]> from the supplied [`Vec`] by calling
    /// [`CpuAccessibleBuffer::from_iter`]
    pub fn create_buffer_from_data<T: 'static + Sized>(&self, data: Vec<T>) -> Arc<CpuAccessibleBuffer<[T]>> {
        CpuAccessibleBuffer::from_iter(self.get_device(), BufferUsage::all(), data.into_iter()).unwrap()
    }
}
