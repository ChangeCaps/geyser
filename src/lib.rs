//! This crate aims to make the use of [`vulkano`] quicker and easier when working on small project.
//! 
//! The first pages you should look at are:
//! 
//! * [`Instance`](instance::Instance)
//! * [`compute`]
//! 
//! # Example
//! ```
//! # #[macro_use]
//! # extern crate geyser;
//! use geyser::instance::Instance;
//! 
//! // Instantiate vulkano
//! let inst = Instance::new();
//! 
//! // Create compute pipeline
//! let pipeline = create_compute_pipeline!(
//!     inst, "
//! #version 450
//! 
//! layout(set = 0, binding = 0) buffer Data {
//!     uint data[];
//! } buf;
//! 
//! void main() {
//!     uint idx = gl_GlobalInvocationID.x;
//! 
//!     buf.data[idx] = idx * 12;
//! }
//! ");
//! 
//! // Create buffer
//! let buf = inst.create_buffer_from_data(vec![0; 69]);
//! 
//! // Create descriptor set
//! let set = create_descriptor_set!([buf], pipeline);
//! 
//! //Run the calculations on the GPU
//! inst.dispatch([69, 1, 1], pipeline.clone(), set.clone());
//! 
//! //Display the results
//! buf.read().expect("Failed to read from buffer")
//!     .iter().enumerate().for_each(|(i, x)| println!("Index: {} equals: {}", i, *x));
//! ```

//#![deny(missing_docs)]
pub extern crate vulkano;
pub extern crate vulkano_shaders;


pub mod instance;
pub mod compute;