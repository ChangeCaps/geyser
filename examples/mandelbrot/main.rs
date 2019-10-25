extern crate geyser;

use geyser::instance::Instance;

fn main() {
    let inst = Instance::new();

    geyser::entire_window_loop!{
        inst, path: "examples/mandelbrot/shader.glsl", {

        }
    };
}