//! This module contains a struct for instancing [`vulkano`]
//! and provides methods for easily creating buffers ect.

use vulkano::{
    device,
    instance,
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

            let mut set = vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start($pipeline.pipeline(), 0);

            Arc::new(set$(.add_buffer($buffer.clone()).unwrap())+.build().unwrap())
        }
    };
}


macro_rules! impl_core {
    ($struct:ident) => {
        impl $crate::core::Core for $struct { 
            fn instance(&self) -> Arc<instance::Instance> {
                self.instance.clone()
            }

            fn device(&self) -> Arc<device::Device> {
                self.device.clone()
            }

            fn queue(&self) -> Arc<device::Queue> {
                self.queue.clone()
            }
        }
    };
}

pub trait Core {

    /// Returns a clone of the [`Arc`]<[`Instance`](instance::Instance)>
    /// inside the [`Core`]
    fn instance(&self) -> Arc<instance::Instance>;

    /// Returns a clone of the [`Arc`]<[`Device`](device::Device)>
    /// inside the [`Core`]
    fn device(&self) -> Arc<device::Device>;


    /// Returns a clone of the [`Arc`]<[`Queue`](device::Device)>
    /// inside the [`Core`]
    fn queue(&self) -> Arc<device::Queue>;

    fn buffer_from_data<T: 'static>(&self, data: Vec<T>) -> Arc<vulkano::buffer::CpuAccessibleBuffer<[T]>> {
        vulkano::buffer::CpuAccessibleBuffer::from_iter(self.device(), 
            vulkano::buffer::BufferUsage::all(), data.into_iter()).unwrap()
    }
}
