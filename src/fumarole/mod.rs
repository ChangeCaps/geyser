use vulkano;
use winit;
use std::sync::Arc;
use vulkano::{
    instance,
    device,
    buffer::*,
};

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
    instance: crate::instance::Instance,
}

impl Fumarole {
    pub fn new() -> Self {
        use vulkano_win::VkSurfaceBuild;

        let instance = instance::Instance::new(None, &vulkano_win::required_extensions(), None).expect("Failed to create instance");

        let physical = instance::PhysicalDevice::enumerate(&instance).next().expect("Fail to create physical instance");

        let queue_family = physical.queue_families()
            .find(|q| q.supports_graphics())
            .expect("No queue families support graphics");

        let (device, mut queues) = device::Device::new(physical, &device::Features::none(), &device::DeviceExtensions::none(),
                                               [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");

        let queue = queues.next().unwrap();

        let mut events_loop = winit::EventsLoop::new();
        let surface = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

        Fumarole {
            events_loop,
            surface,
            instance: crate::instance::Instance {
                instance,
                device,
                queue,
            }
        }
    }
}

impl_instance_functions!(Fumarole, instance);