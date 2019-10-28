#[macro_use]
extern crate geyser;

fn main() {
    use geyser::cryo::Cryo;
    use geyser::core::*;
 
    let inst = Cryo::new();
 
    let pipeline = compute_pipeline!(
        inst, 
        src: "
#version 450
 
layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;
 
void main() {
    uint idx = gl_GlobalInvocationID.x;
 
    buf.data[idx] = idx * 12;
}
    ");
 
    let buf = inst.buffer_from_data(vec![0; 69]);
 
    let set = descriptor_set!([buf], pipeline);
 
    pipeline.dispatch([69, 1, 1], set.clone());
 
    buf.read().expect("Failed to read from buffer")
        .iter().enumerate().for_each(|(i, x)| println!("Index: {} equals: {}", i, *x));
}
