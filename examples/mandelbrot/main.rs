extern crate geyser;

fn main() {
    let inst = Instance::new();

    geyser::entire_window_loop!{
        inst, path: "examples/mandelbrot/shader.glsl", {

        }
    };
}