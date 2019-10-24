#[macro_use]
extern crate geyser;

use geyser::instance;
use geyser::instance::Instance;

fn main() {
    let inst = Instance::new();

    let pipeline = create_compute_pipeline!(inst, "
#version 450

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    buf.data[gl_GlobalInvocationID.x] *= 12;
}
    ");

    let a = inst.create_buffer_from_data(vec![100; 200]);

    let set = create_descriptor_set!([a], pipeline);

    inst.dispatch([200, 1, 1], pipeline.clone(), set.clone());

    let data = a.read().unwrap();

    for (n, val) in data.iter().enumerate() {
        println!("{}, {}", n, val);
    }
}
