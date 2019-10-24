//! This module contains macros and structs for GPGPU

use instance::Instance;
use std::sync::Arc;
use vulkano::{
    descriptor::*,
    pipeline::*,
};

/// Creates a [`Arc`](std::sync::Arc)<[`ComputePipeline`](vulkano::pipeline::ComputePipeline)>.
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
/// let pipeline = create_compute_pipeline!(
///     inst, "
/// #version 450
/// 
/// layout(set = 0, binding = 0) buffer Data {
///     uint data[];
/// } buf;
/// 
/// void main() {
///     let idx = gl_GlobalInvocationID.x;
/// 
///     buf.data[idx] = idx * 12;
/// }
/// ");
/// ```
#[macro_export]
macro_rules! create_compute_pipeline {
    ($instance:expr, $source_code:expr) => {
        {
            use geyser::instance;
            use geyser::vulkano_shaders;
            use std::sync::Arc;
            use geyser::vulkano::pipeline::ComputePipeline;

            mod cs {
                vulkano_shaders::shader!{
                    ty: "compute",
                    src: $source_code,
                }
            }

            Arc::new(ComputePipeline::new($instance.get_device(), 
                     &cs::Shader::load($instance.get_device()).unwrap().main_entry_point(), 
                     &()).unwrap())
        }
    };
}

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
/// let pipeline = create_compute_pipeline!(
///     inst, "
/// #version 450
/// 
/// layout(set = 0, binding = 0) buffer Data {
///     uint data[];
/// } buf;
/// 
/// void main() {
///     let idx = gl_GlobalInvocationID.x;
/// 
///     buf.data[idx] = idx * 12;
/// }
/// ");
/// 
/// let buf = inst.create_buffer_from_data(vec![42; 69]);
/// 
/// let set = create_descriptor_set!([buf], pipeline);
/// ```
#[macro_export]
macro_rules! create_descriptor_set {
    ([$($buffer:expr),+], $pipeline:expr) => {
        {
            use std::sync::Arc;

            let mut set = vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start($pipeline.clone(), 0);

            Arc::new(set$(.add_buffer($buffer.clone()).unwrap())+.build().unwrap())
        }
    };
}

impl Instance {
    /// Creates a [`AutoCommandBuffer`](vulkano::command_buffer::AutoCommandBuffer), calls 
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
            self.get_device(), self.get_queue().family()).unwrap()
                .dispatch(size, pipeline.clone(), set.clone(), ()).unwrap()
                .build().unwrap();

        let finished = command_buffer.execute(self.get_queue()).unwrap();

        finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    }
}