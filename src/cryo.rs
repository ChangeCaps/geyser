//! This module contains macros and structs for GPGPU

use core::Core;
use std::sync::Arc;
use vulkano::{
    instance,
    device,
    descriptor::*,
    pipeline::*,
    buffer::*,
};









/// Creates an [`Arc`](std::sync::Arc)<[`ComputePipeline`](vulkano::pipeline::ComputePipeline)>.
/// It takes the code for the shader as literate sting and an [`Instance`](instance::Instance).
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
/// ```
#[macro_export]
macro_rules! compute_pipeline {
    ($instance:expr, $tt:tt: $source_code:expr) => {
        {
            use geyser::core::Core;
            use geyser::cryo::Pipeline;
            use geyser::vulkano_shaders;
            use std::sync::Arc;
            use geyser::vulkano::pipeline::ComputePipeline;

            mod cs {
                vulkano_shaders::shader!{
                    ty: "compute",
                    $tt: $source_code,
                }
            }

            let pipeline = Arc::new(ComputePipeline::new($instance.device(), 
                     &cs::Shader::load($instance.device()).unwrap().main_entry_point(), 
                     &()).unwrap());

            Pipeline::new(pipeline, $instance.device(), $instance.queue())
        }
    };
}













/// This is a struct that holds an [`Arc`]<[`Instance`]>, [`Arc`]<[`Device`](device::Device)> and an [`Arc`]<[`Queue`](device::Device)>.
/// This serves the purpose of making it easier to create everything needed for your GPU calculations.
/// Note that you should try to **never** call [`Cryo::new`] more than once!
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
pub struct Cryo {
    instance: Arc<instance::Instance>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
}

#[allow(dead_code)]
impl Cryo {
    /// Creates a new [`Instance`].
    /// 
    /// It uses the first [`QueueFamily`](instance::QueueFamily) that supports graphics and the first [`Queue`](device::Queue) in that [`QueueFamily`](instance::QueueFamily)
    pub fn new() -> Cryo {
        let instance = instance::Instance::new(None, &instance::InstanceExtensions::none(), None).expect("Failed to create instance");

        let physical = instance::PhysicalDevice::enumerate(&instance).next().expect("Fail to create physical instance");

        let queue_family = physical.queue_families()
            .find(|q| q.supports_graphics())
            .expect("No queue families support graphics");

        let (device, mut queues) = device::Device::new(physical, &device::Features::none(), &device::DeviceExtensions::none(),
                                               [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");

        let queue = queues.next().unwrap();


        Cryo {
            instance,
            device,
            queue,
        }
    }
}

impl_core!(Cryo);











// Pipeline struct

#[derive(Clone)]
pub struct Pipeline<C> 
    where Arc<ComputePipeline<C>>: Clone
{
    pipeline: Arc<ComputePipeline<C>>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
}

impl<C: 'static> Pipeline<C> {

    pub fn new(pipeline: Arc<ComputePipeline<C>>, device: Arc<device::Device>, queue: Arc<device::Queue>) -> Self {
        Pipeline {
            pipeline,
            device,
            queue,
        }
    }

    pub fn pipeline(&self) -> Arc<ComputePipeline<C>> {
        self.pipeline.clone()
    }

    /// Creates an [`AutoCommandBuffer`](vulkano::command_buffer::AutoCommandBuffer), calls 
    /// [`AutoCommandBuffer::execute`](vulkano::command_buffer::CommandBuffer::execute) on it and waits for it to finish. 
    /// 
    /// This **blocks** until the calculation is finished.
    pub fn dispatch<L: 'static, R: 'static>(&self, size: [u32; 3], 
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
            self.device.clone(), self.queue.clone().family()).unwrap()
                .dispatch(size, self.pipeline.clone(), set.clone(), ()).unwrap()
                .build().unwrap();

        let finished = command_buffer.execute(self.queue.clone()).unwrap();

        finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    }
}
