//! This crate aims to make the use of [`vulkano`] quicker and easier when working on a smaller project.
//! 
//! 
//! # Example
//! ```
//! # #[macro_use]
//! # extern crate geyser;
//! use geyser::Cryo;
//! 
//! // Instantiate vulkano
//! let cryo = Cryo::new();
//! 
//! // Create compute pipeline
//! let pipeline = compute_pipeline!(
//!     inst, 
//!     src: "
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
//! let buf = cryo.buffer_from_data(vec![0; 69]).expect("Failed to create buffer");
//! 
//! // Create descriptor set
//! let set = descriptor_set!([buf], pipeline);
//! 
//! // Dispatch
//! cryo.dispatch([69, 1, 1], pipeline.clone(), set.clone());
//! 
//! // Display the results
//! buf.read().expect("Failed to read from buffer")
//!     .iter().enumerate().for_each(|(i, x)| println!("Index: {} equals: {}", i, *x));
//! ```

//#![deny(missing_docs)]
pub extern crate vulkano;
pub extern crate vulkano_shaders;

#[macro_use]
mod cryo;
pub use cryo::*;
