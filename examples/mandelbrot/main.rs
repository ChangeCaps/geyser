extern crate geyser;

use geyser::core::*;

fn main() {
    let inst = geyser::fumarole::Fumarole::new();

    geyser::single_pass!{
        inst, 
        path: "examples/mandelbrot/shader.glsl", 
        geyser::fumarole::cover_screen()    
    };
}
