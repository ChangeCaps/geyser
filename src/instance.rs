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

            let mut set = vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start($pipeline.clone(), 0);

            Arc::new(set$(.add_buffer($buffer.clone()).unwrap())+.build().unwrap())
        }
    };
}


macro_rules! impl_instance_functions {
    ($struct:ident, $field:ident) => {
        impl $struct {
            /// Returns a clone of the [`Arc`]<[`Instance`](instance::Instance)>
            /// inside the [`Wrapper`]
            pub fn instance(&self) -> Arc<instance::Instance> {
                self.$field.instance.clone()
            }

            /// Returns a clone of the [`Arc`]<[`Device`](device::Device)>
            /// inside the [`Wrapper`]
            pub fn device(&self) -> Arc<device::Device> {
                self.$field.device.clone()
            }

            /// Returns a clone of the [`Arc`]<[`Queue`](device::Device)>
            /// inside the [`Wrapper`]
            pub fn queue(&self) -> Arc<device::Queue> {
                self.$field.queue.clone()
            }

            /// Creates an [`Arc`]<[`CpuAccessibleBuffer`]> from the supplied [`Vec`]
            pub fn buffer_from_data<T: 'static + Sized>(&self, data: Vec<T>) -> Arc<CpuAccessibleBuffer<[T]>> {
                CpuAccessibleBuffer::from_iter(self.device(), BufferUsage::all(), data.into_iter()).unwrap()
            }
        }
    };
}

pub struct Instance {
    pub(crate) instance: Arc<instance::Instance>,
    pub(crate) device: Arc<device::Device>,
    pub(crate) queue: Arc<device::Queue>,
}

