//! This module contains macros and structs for GPGPU

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
            use geyser::instance;
            use geyser::vulkano_shaders;
            use std::sync::Arc;
            use geyser::vulkano::pipeline::ComputePipeline;

            mod cs {
                vulkano_shaders::shader!{
                    ty: "compute",
                    $tt: $source_code,
                }
            }

            Arc::new(ComputePipeline::new($instance.device(), 
                     &cs::Shader::load($instance.device()).unwrap().main_entry_point(), 
                     &()).unwrap())
        }
    };
}