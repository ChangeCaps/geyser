use vulkano;
use winit;
use std::sync::Arc;
use vulkano::{
    instance,
    device,
};

pub mod macros;
pub mod default;
pub use self::default::*;

///This 
#[derive(Default, Copy, Clone)]
pub struct Vertex2 {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex2, position);


impl Vertex2 {
    pub const fn new(x: f32, y: f32) -> Self {
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
    event_loop: winit::event_loop::EventLoop<()>,
    surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    swapchain: Arc<vulkano::swapchain::Swapchain<winit::window::Window>>,
    instance: Arc<instance::Instance>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
}

impl Fumarole {
    pub fn new(dimensions: [u32; 2]) -> Self {
        use vulkano_win::VkSurfaceBuild;

        let instance = instance::Instance::new(None, &vulkano_win::required_extensions(), None).expect("Failed to create instance");

        let physical = instance::PhysicalDevice::enumerate(&instance).next().expect("Fail to create physical instance");

        let queue_family = physical.queue_families()
            .find(|q| q.supports_graphics())
            .expect("No queue families support graphics");

        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none() 
        };
        
        let (device, mut queues) = device::Device::new(physical, physical.supported_features(), &device_ext,
                                               [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");

        let queue = queues.next().unwrap();

        let event_loop = winit::event_loop::EventLoop::new();
        let surface = winit::window::WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

        let caps = surface.capabilities(physical).
            expect("failed to get surface capabilities");

        let dimensions = caps.current_extent.unwrap_or(dimensions);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let (swapchain, images) = vulkano::swapchain::Swapchain::new(
            device.clone(), surface.clone(), caps.min_image_count, format, dimensions, 1,
            caps.supported_usage_flags, &queue, vulkano::swapchain::SurfaceTransform::Identity,
            alpha, vulkano::swapchain::PresentMode::Fifo, true, None)
                .expect("failed to create swapchain");

        Fumarole {
            event_loop,
            surface,
            swapchain,
            instance,
            device,
            queue,
        }
    }
}

impl_core!(Fumarole);
