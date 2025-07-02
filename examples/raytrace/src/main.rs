use geyser::*;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).unwrap();
}

struct App {
    instance: Instance,
    device: Device,
    queue: Queue,
}

impl App {
    fn new<T>(event_loop: &EventLoop<T>) -> Self {
        let display_handle = event_loop.display_handle().unwrap().as_raw();

        let entry = Entry::linked();
        let instance = entry.create_instance(&InstanceDescriptor {
            max_api_version: Some(Version::V1_3),
            enabled_extensions: Instance::required_surface_extensions(display_handle),
            ..Default::default()
        });

        let physical_devices = instance.enumerate_physical_devices();
        let physical_device = physical_devices
            .iter()
            .find(|p| p.properties().device_kind == PhysicalDeviceKind::DiscreteGpu)
            .expect("Suitable physical device found");

        let queue_family = physical_device
            .queue_families()
            .iter()
            .position(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .expect("Suitable queue family found");

        let (device, queues) = physical_device.create_device(&DeviceDescriptor {
            queue_families: &[QueueFamilyDescriptor {
                family_index: queue_family as u32,
                priorities: &[1.0],
            }],
            enabled_extensions: DeviceExtensions {
                khr_swapchain: true,
                ..Default::default()
            },
            ..Default::default()
        });

        let queue = queues.into_iter().flatten().next().unwrap();

        Self {
            instance,
            device,
            queue,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes().with_title("Raytrace Example");

        let window = event_loop.create_window(attributes).unwrap();

        let surface = unsafe {
            self.instance.create_surface(
                event_loop.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
            )
        };

        let formats = self.device.physical().get_surface_formats(&surface);
        let present_modes = self.device.physical().get_surface_present_modes(&surface);
        let capabilities = self.device.physical().get_surface_capabilities(&surface);

        tracing::info!("Surface capabilities: {:#?}", capabilities);
        tracing::info!("Surface formats: {:#?}", formats);
        tracing::info!("Surface present modes: {:#?}", present_modes);

        let size = window.inner_size();

        let swapchain = self.device.create_swapchain(
            surface,
            &SwapchainDescriptor {
                min_image_count: capabilities.min_image_count,
                image_extent: Extent2d {
                    width: size.width,
                    height: size.height,
                },
                image_usage: ImageUsages::COLOR_ATTACHMENT | ImageUsages::TRANSFER_SRC,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                // Handle redraw requests here
            }

            _ => {}
        }
    }
}
