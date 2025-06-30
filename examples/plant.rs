use std::error::Error;

use geyser::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::info;
use tracing_subscriber::EnvFilter;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let event_loop = EventLoop::new()?;

    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}

struct App {
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    queue: Queue,
    command_pool: CommandPool,
    image_available: Vec<Semaphore>,
    render_finished: Vec<Semaphore>,
    in_flight_fence: Vec<Fence>,
    command_buffers: Vec<CommandBuffer>,
    current_frame: usize,
    window: Option<(Swapchain, Window)>,
}

impl App {
    fn new<T>(event_loop: &EventLoop<T>) -> Self {
        let display_handle = event_loop
            .display_handle()
            .expect("Failed to get display handle");

        let extensions = Instance::required_window_extensions(&display_handle)
            .expect("Failed to get required window extensions");

        info!("Supported instance extensions:");
        for ext in Instance::extension_properties() {
            info!("\t{} (version {})", ext.name, ext.version);
        }

        let instance = Instance::new(&InstanceDescriptor {
            api_version: Version::V1_3,
            layers: &[InstanceLayer::KHRONOS_VALIDATION],
            extensions: &extensions,
            ..Default::default()
        });

        let physical = instance
            .physical_devices()
            .into_iter()
            .find(|physical| physical.is_discrete())
            .expect("No discrete GPU found");

        info!("Using physical device: {}", physical.name());
        info!("Supported device extensions:");

        for ext in physical.extensions() {
            info!("\t{} (version {})", ext.name, ext.version);
        }

        let (device, mut queues) = physical.create_device(&DeviceDescriptor {
            queues: &[QueueDescriptor::new(0, &[1.0])],
            extensions: &[
                DeviceExtension::KHR_SWAPCHAIN,
                DeviceExtension::KHR_ACCELERATION_STRUCTURE,
                DeviceExtension::KHR_DEFERRED_HOST_OPERATIONS,
                DeviceExtension::KHR_BUFFER_DEVICE_ADDRESS,
                DeviceExtension::KHR_RAY_TRACING_PIPELINE,
                DeviceExtension::KHR_DYNAMIC_RENDERING,
                DeviceExtension::EXT_DESCRIPTOR_INDEXING,
            ],
            dynamic_rendering: true,
            acceleration_structure: true,
            buffer_device_address: true,
        });

        let blas_sizes = device.get_blas_build_sizes(&BlasDescriptor {
            flags: AccelBuildFlags::empty(),
            geometries: &[BlasGeometry {
                data: BlasGeometryData::Aabbs {
                    stride: 32,
                    max_count: 16,
                },
                flags: GeometryFlags::OPAQUE,
            }],
        });

        info!("Blas: {:?}", blas_sizes);

        let blas_buffer = device.create_buffer(&BufferDescriptor {
            size: blas_sizes.accel_size,
            usages: BufferUsages::ACCELERATION_STRUCTURE_STORAGE,
            ..Default::default()
        });
        device.allocate_buffer_memory(
            &blas_buffer,
            MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
            MemoryAllocateFlags::empty(),
        );

        let blas = device.create_blas(&blas_buffer, 0, blas_sizes.accel_size);

        let tlas_sizes = device.get_tlas_build_sizes(&TlasDescriptor {
            flags: AccelBuildFlags::empty(),
            max_instance_count: 1,
        });

        info!("Tlas: {:?}", tlas_sizes);

        let tlas_buffer = device.create_buffer(&BufferDescriptor {
            size: tlas_sizes.accel_size,
            usages: BufferUsages::ACCELERATION_STRUCTURE_STORAGE,
            ..Default::default()
        });
        device.allocate_buffer_memory(
            &tlas_buffer,
            MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
            MemoryAllocateFlags::empty(),
        );

        let geometry_buffer = device.create_buffer(&BufferDescriptor {
            size: 32 * 16, // 16 AABBs, each 32 bytes
            usages: BufferUsages::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY
                | BufferUsages::STORAGE_BUFFER
                | BufferUsages::SHADER_DEVICE_ADDRESS,
            ..Default::default()
        });
        let mut geometry_memory = device.allocate_buffer_memory(
            &geometry_buffer,
            MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
            MemoryAllocateFlags::DEVICE_ADDRESS,
        );

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            size: 64,
            usages: BufferUsages::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY
                | BufferUsages::STORAGE_BUFFER
                | BufferUsages::SHADER_DEVICE_ADDRESS,
            ..Default::default()
        });
        let mut instance_memory = device.allocate_buffer_memory(
            &instance_buffer,
            MemoryProperties::HOST_VISIBLE | MemoryProperties::HOST_COHERENT,
            MemoryAllocateFlags::DEVICE_ADDRESS,
        );

        let blas_scratch = device.create_buffer(&BufferDescriptor {
            size: blas_sizes.build_scratch_size,
            usages: BufferUsages::ACCELERATION_STRUCTURE_STORAGE
                | BufferUsages::STORAGE_BUFFER
                | BufferUsages::SHADER_DEVICE_ADDRESS,
            ..Default::default()
        });
        device.allocate_buffer_memory(
            &blas_scratch,
            MemoryProperties::DEVICE_LOCAL,
            MemoryAllocateFlags::DEVICE_ADDRESS,
        );

        let tlas_scratch = device.create_buffer(&BufferDescriptor {
            size: tlas_sizes.build_scratch_size,
            usages: BufferUsages::ACCELERATION_STRUCTURE_STORAGE
                | BufferUsages::STORAGE_BUFFER
                | BufferUsages::SHADER_DEVICE_ADDRESS,
            ..Default::default()
        });
        device.allocate_buffer_memory(
            &tlas_scratch,
            MemoryProperties::DEVICE_LOCAL,
            MemoryAllocateFlags::DEVICE_ADDRESS,
        );

        instance_memory.map(0..64).fill(0);
        geometry_memory.map(0..32 * 16).fill(0);

        let tlas = device.create_tlas(&tlas_buffer, 0, tlas_sizes.accel_size);

        let queue = queues[0].remove(0);

        let command_pool = queue.create_command_pool(CommandPoolFlags::RESET);

        let mut command_buffer = command_pool.allocate_command_buffer(CommandBufferLevel::Primary);

        let mut encoder = command_buffer.begin(CommandBufferUsages::ONE_TIME_SUBMIT);

        encoder.build_acceleration_structures(
            &[BlasBuildDescriptor {
                blas: &blas,
                mode: AccelBuildMode::Build,
                geometries: &[BlasBuildGeometry {
                    data: BlasBuildGeometryData::Aabbs {
                        buffer: &geometry_buffer,
                        offset: 0,
                        stride: 32,
                        count: 16,
                    },
                    flags: GeometryFlags::OPAQUE,
                }],
                scratch: &blas_scratch,
                flags: AccelBuildFlags::empty(),
            }],
            &[TlasBuildDescriptor {
                tlas: &tlas,
                mode: AccelBuildMode::Build,
                buffer: &instance_buffer,
                offset: 0,
                count: 0,
                scratch: &tlas_scratch,
            }],
        );

        encoder.end();

        queue.submit(
            &[Submit {
                command_buffers: &[&command_buffer],
                ..Default::default()
            }],
            None,
        );

        App {
            instance,
            physical_device: physical,
            device,
            queue,
            command_pool,
            image_available: Vec::new(),
            render_finished: Vec::new(),
            in_flight_fence: Vec::new(),
            command_buffers: Vec::new(),
            current_frame: 0,
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();

        let display_handle = window
            .display_handle()
            .expect("Failed to get display handle");

        let window_handle = window
            .window_handle() //
            .expect("Failed to get window handle");

        let surface = unsafe {
            self.instance
                .create_surface(&display_handle, &window_handle)
                .expect("Failed to create surface")
        };

        if !(self.physical_device).is_surface_supported(&surface, self.queue.family()) {
            panic!("Surface is not supported by the physical device");
        }

        let swapchain = self.device.create_swapchain(
            surface,
            &SwapchainDescriptor {
                min_image_count: 4,
                image_extent: Extent2d {
                    width: window.inner_size().width,
                    height: window.inner_size().height,
                },
                image_usage: ImageUsages::COLOR_ATTACHMENT,
                ..Default::default()
            },
        );

        for _ in swapchain.images() {
            let image_available = self.device.create_semaphore();
            let render_finished = self.device.create_semaphore();
            let in_flight_fence = self.device.create_fence(true);

            self.image_available.push(image_available);
            self.render_finished.push(render_finished);
            self.in_flight_fence.push(in_flight_fence);
        }

        self.command_buffers = self
            .command_pool
            .allocate_command_buffers(swapchain.images().len() as u32, CommandBufferLevel::Primary);

        self.window = Some((swapchain, window));
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
                let Some((ref swapchain, ref _window)) = self.window else {
                    return;
                };

                let image_available = &self.image_available[self.current_frame];
                let render_finished = &self.render_finished[self.current_frame];
                let in_flight_fence = &self.in_flight_fence[self.current_frame];
                let command_buffer = &mut self.command_buffers[self.current_frame];

                in_flight_fence.wait(None);
                in_flight_fence.reset();

                let swapchain_image = swapchain
                    .aquire_next_image(None, Some(image_available), None)
                    .expect("Failed to acquire next image");

                let image_view = swapchain_image.image.create_view(&Default::default());

                let mut encoder = command_buffer.begin(CommandBufferUsages::ONE_TIME_SUBMIT);

                encoder.pipeline_barrier(&PipelineBarrier {
                    image_barriers: &[ImageBarrier {
                        old_layout: ImageLayout::Undefined,
                        new_layout: ImageLayout::ColorAttachmentOptimal,
                        aspects: ImageAspects::COLOR,
                        ..ImageBarrier::default(&swapchain_image)
                    }],
                    ..Default::default()
                });

                let pass = encoder.begin_rendering(&RenderingInfo {
                    area: Rect2d::from(swapchain_image.extent().to_2d()),
                    color_attachments: &[RenderingColorAttachment {
                        clear_value: [0.0, 1.0, 1.0, 1.0],
                        ..RenderingColorAttachment::default(&image_view)
                    }],
                    ..Default::default()
                });

                pass.end();

                encoder.pipeline_barrier(&PipelineBarrier {
                    image_barriers: &[ImageBarrier {
                        old_layout: ImageLayout::ColorAttachmentOptimal,
                        new_layout: ImageLayout::PresentSrc,
                        aspects: ImageAspects::COLOR,
                        ..ImageBarrier::default(&swapchain_image)
                    }],
                    ..Default::default()
                });

                encoder.end();

                self.queue.submit(
                    &[Submit {
                        wait_semaphores: &[WaitSemaphore {
                            semaphore: image_available,
                            dst_stage_mask: PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                        }],
                        command_buffers: &[command_buffer],
                        signal_semaphores: &[render_finished],
                    }],
                    Some(&self.in_flight_fence[self.current_frame]),
                );

                self.queue
                    .present(
                        &[swapchain_image],
                        &[&self.render_finished[self.current_frame]],
                    )
                    .expect("Failed to present image");

                self.current_frame = (self.current_frame + 1) % self.image_available.len();
            }

            WindowEvent::Resized(new_size) => {
                if let Some((ref mut swapchain, ref window)) = self.window {
                    self.device.wait_idle();

                    swapchain.recreate(&SwapchainDescriptor {
                        min_image_count: 4,
                        image_extent: Extent2d {
                            width: new_size.width,
                            height: new_size.height,
                        },
                        image_usage: ImageUsages::COLOR_ATTACHMENT,
                        ..Default::default()
                    });

                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}
