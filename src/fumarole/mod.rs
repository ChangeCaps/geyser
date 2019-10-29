use vulkano;
use winit;
use std::sync::Arc;
use crate::core::Core;
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
    pub(crate) event_loop: winit::event_loop::EventLoop<()>,
    pub(crate) surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    pub(crate) swapchain: Arc<vulkano::swapchain::Swapchain<winit::window::Window>>,
    pub(crate) images: Vec<Arc<vulkano::image::SwapchainImage<winit::window::Window>>>,
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
            images,
            instance,
            device,
            queue,
        }
    }
}

impl_core!(Fumarole);







pub struct Pipeline<A, B, C, R> {
    pipeline: Arc<vulkano::pipeline::GraphicsPipeline<A, B, C>>,
    render_pass: Arc<vulkano::framebuffer::RenderPass<R>>,
}

impl<A, B, C, R: 'static> Pipeline<A, B, C, R> 
    where vulkano::framebuffer::RenderPass<R>: Send + Sync + vulkano::framebuffer::RenderPassAbstract
{
    pub fn new(pipeline: Arc<vulkano::pipeline::GraphicsPipeline<A, B, C>>, 
        render_pass: Arc<vulkano::framebuffer::RenderPass<R>>) -> Self {
        Self {
            pipeline,
            render_pass,
        }
    }

    pub fn run_with_loop<E>(&mut self, fumarole: &mut Fumarole, each_frame: fn (winit::event::Event<E>) -> bool) {
        let mut dynamic_state = vulkano::command_buffer::DynamicState {line_width: None, scissors: None, viewports: None};
        
        // Setting up framebuffers for each image
        let mut framebuffers = window_size_dependent_setup(&fumarole.images, self.render_pass.clone(), &mut dynamic_state);
        
        let mut recreate_swapchain = false; 
        
        let mut previous_frame_end = Box::new(vulkano::sync::now(fumarole.device())) as Box<dyn vulkano::sync::GpuFuture>;
    
        //Here is the main loop
        loop {
            previous_frame_end.cleanup_finished();

            if recreate_swapchain {
                let dimensions: (u32, u32) = fumarole.surface.window().inner_size().to_physical(
                    fumarole.surface.window().hidpi_factor()).into();

                let dimensions = [dimensions.0, dimensions.1];

                let (new_swapchain, new_images) = match fumarole.swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => continue,
                    Err(err) => panic!("{:?}", err),
                };

                framebuffers = window_size_dependent_setup(&new_images, self.render_pass.clone(), &mut dynamic_state);

                fumarole.swapchain = new_swapchain;
                fumarole.images = new_images;

                recreate_swapchain = false;
            }

            let (image_num, aquire_future) = match vulkano::swapchain::acquire_next_image(fumarole.swapchain.clone(), None) {
                Ok(r) => r,
                Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                },
                Err(err) => panic!("{:?}", err),
            };



            let clear_values = vec!([0.0, 0.0, 0.0, 1.0].into());


            let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::
                primary_one_time_submit(fumarole.device(), fumarole.queue().family()).unwrap()
                    .begin_render_pass(framebuffers[image_num].clone(), false, clear_values).unwrap()
                    .draw(self.pipeline.clone(), &dynamic_state, 
        }
    }
}


fn window_size_dependent_setup(
    images: &[Arc<vulkano::image::swapchain::SwapchainImage<winit::window::Window>>],
    render_pass: Arc<dyn vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut vulkano::command_buffer::DynamicState
) -> Vec<Arc<dyn vulkano::framebuffer::FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = vulkano::pipeline::viewport::Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn vulkano::framebuffer::FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}
