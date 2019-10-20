#[macro_use]
extern crate geyser;

use geyser::instance;
use geyser::instance::Instance;

fn main() {
    let inst = Instance::new();

    let pipeline = create_compute_pipeline!(inst, "
#version 450

void main() {

}
    ");

    let a = inst.create_buffer_from_data(vec![0; 200]);

    
}