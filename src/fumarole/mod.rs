use vulkano;
use winit;
use std::sync::Arc;


///This 
#[derive(Default, Copy, Clone)]
pub struct Vertex2 {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex2, position);

impl Vertex2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vertex2 {
            position: [x, y],
        }
    }
}

impl From<[f32; 2]> for Vertex2 {
    fn from(vert: [f32; 2]) -> Self {
        Vertex2 {
            position: vert,
        }
    }
}

pub struct Fumarole {
    events_loop: winit::EventsLoop,
    surface: Arc<vulkano::swapchain::Surface<winit::Window>>,
}