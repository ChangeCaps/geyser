#[macro_use]
extern crate geyser;

fn main() {
    use geyser::instance::Instance;
 
    let inst = Instance::new();
 
    let pipeline = create_compute_pipeline!(
        inst, "
#version 450
 
layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;
 
void main() {
    uint idx = gl_GlobalInvocationID.x;
 
    buf.data[idx] = idx * 12;
}
    ");
 
    let buf = inst.create_buffer_from_data(vec![0; 69]);
 
    let set = create_descriptor_set!([buf], pipeline);
 
    inst.dispatch([69, 1, 1], pipeline.clone(), set.clone());
 
    buf.read().expect("Failed to read from buffer")
        .iter().enumerate().for_each(|(i, x)| println!("Index: {} equals: {}", i, *x));
}
