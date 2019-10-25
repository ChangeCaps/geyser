//! This module contains a struct for instancing [`vulkano`]
//! and provides methods for easily creating buffers ect.

use vulkano::{
    device,
    instance,
    buffer::*,
    descriptor::*,
    pipeline::*,
};
use std::sync::Arc;



/// Creates an [`Arc`](std::sync::Arc)<[`PersistantDescriptorSet`](vulkano::pipeline::ComputePipeline)> from list of [`buffer`](vulkano::buffer) and a [`pipeline`](vulkano::pipeline)
/// 
/// # Example
/// ```
/// # #[macro_use]
/// # extern crate geyser;
/// use geyser::instance::Instance;
/// 
/// let inst = Instance::new();
/// 
/// let pipeline = compute_pipeline!(
///     inst, 
///     src: "
/// #version 450
/// 
/// layout(set = 0, binding = 0) buffer Data {
///     uint data[];
/// } buf;
/// 
/// void main() {
///     uint idx = gl_GlobalInvocationID.x;
/// 
///     buf.data[idx] = idx * 12;
/// }
/// ");
/// 
/// let buf = inst.buffer_from_data(vec![42; 69]);
/// 
/// let set = descriptor_set!([buf], pipeline);
/// ```
#[macro_export]
macro_rules! descriptor_set {
    ([$($buffer:expr),+], $pipeline:expr) => {
        {
            use std::sync::Arc;

            let mut set = vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start($pipeline.clone(), 0);

            Arc::new(set$(.add_buffer($buffer.clone()).unwrap())+.build().unwrap())
        }
    };
}

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
/// let buf = inst.buffer_from_data(vec![42; 69]);
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
    pub fn instance(&self) -> Arc<instance::Instance> {
        self.instance.clone()
    }

    /// Returns a clone of the [`Arc`]<[`Device`](device::Device)>
    /// inside the [`Instance`]
    pub fn device(&self) -> Arc<device::Device> {
        self.device.clone()
    }

    /// Returns a clone of the [`Arc`]<[`Queue`](device::Device)>
    /// inside the [`Instance`]
    pub fn queue(&self) -> Arc<device::Queue> {
        self.queue.clone()
    }

    //Stuff

    /// Creates an [`Arc`]<[`CpuAccessibleBuffer`]> from the supplied [`Vec`] by calling
    /// [`CpuAccessibleBuffer::from_iter`]
    pub fn buffer_from_data<T: 'static + Sized>(&self, data: Vec<T>) -> Arc<CpuAccessibleBuffer<[T]>> {
        CpuAccessibleBuffer::from_iter(self.device(), BufferUsage::all(), data.into_iter()).unwrap()
    }

    /// Creates an [`AutoCommandBuffer`](vulkano::command_buffer::AutoCommandBuffer), calls 
    /// [`AutoCommandBuffer::execute`](vulkano::command_buffer::CommandBuffer::execute) on it and waits for it to finish. 
    /// 
    /// This **blocks** until the calculation is finished.
    pub fn dispatch<L: 'static, R: 'static, C: 'static>(&self, size: [u32; 3], pipeline: Arc<ComputePipeline<C>>, 
        set: Arc<descriptor_set::PersistentDescriptorSet<L, R>>)
        
        where L: Send + Sync,
              R: Send + Sync,
              C: Send + Sync,
              ComputePipeline<C>: ComputePipelineAbstract,
              descriptor_set::PersistentDescriptorSet<L, R>: DescriptorSet,
    {
        use vulkano::command_buffer::CommandBuffer;
        use vulkano::sync::GpuFuture;

        let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::new(
            self.device(), self.queue().family()).unwrap()
                .dispatch(size, pipeline.clone(), set.clone(), ()).unwrap()
                .build().unwrap();

        let finished = command_buffer.execute(self.queue()).unwrap();

        finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    }
}
