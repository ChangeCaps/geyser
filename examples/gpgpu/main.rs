#[macro_use]
extern crate geyser;

geyser::shader! {
    name: multiply, 
    path: "examples/gpgpu/shader.glsl"
}

fn main() {
    use geyser::Cryo;
 
    let inst = Cryo::new();

    let pipeline = compute_pipeline!(
        inst, 
        multiply    
    );

    let pc = PCData {
        multiple: 24,
    };

    let buf = inst.buffer_from_data(vec![0; 69]).unwrap();
 
    let set = descriptor_set!([buf], pipeline);
 
    pipeline.dispatch([69, 1, 1], set.clone(), pc);
 
    buf.read().expect("Failed to read from buffer")
        .iter().enumerate().for_each(|(i, x)| println!("Index: {} equals: {}", i, *x));
}
