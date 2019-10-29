extern crate geyser;

use geyser::core::*;

fn main() {
    let mut inst = geyser::fumarole::Fumarole::new([1200, 600]);

    let mut pipeline = geyser::single_pass_pipeline!{
        inst, 
        path: "examples/mandelbrot/shader.glsl", 
        geyser::fumarole::cover_screen()    
    };

    pipeline.run_with_loop(&mut inst, |event: geyser::winit::event::Event<i32>| {
        true
    });
}
