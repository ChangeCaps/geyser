use vulkano::{
    device,
    instance,
    buffer::*,
    pipeline::*,
    descriptor::*,
};
use std::sync::Arc;

#[allow(dead_code)]
pub struct Instance {
    instance: Arc<instance::Instance>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
}

#[macro_export]
macro_rules! create_compute_pipeline {
    ($instance:expr, $source_code:expr) => {
        {
            use instance;
            use vulkano_shaders;
            use std::sync::Arc;
            use vulkano::pipeline::ComputePipeline;

            mod cs {
                vulkano_shaders::shader!{
                    ty: "compute",
                    src: $source_code,
                }
            }

            Arc::new(ComputePipeline::new($instance.get_device(), &cs::Shader::load($instance.get_device()).unwrap().main_entry_point(), &()).unwrap())
        }
    };
}

#[allow(dead_code)]
impl Instance {
    //Creation

    pub fn new() -> Instance {
        let instance = instance::Instance::new(None, &instance::InstanceExtensions::none(), None).expect("Failed to create instance");

        let physical = instance::PhysicalDevice::enumerate(&instance).next().expect("Fail to create physical instance");

        let queue_family = physical.queue_families()
            .find(|q| q.supports_graphics())
            .expect("No queue families support graphics");

        let (device, mut queues) = device::Device::new(physical, &device::Features::none(), &device::DeviceExtensions::none(),
                                               [(queue_family, 0.5)].iter().cloned()).expect("failed to create device");

        let queue = queues.next().unwrap();


        Instance {
            instance,
            device,
            queue,
        }
    }

    //Interfaces

    pub fn get_instance(&self) -> Arc<instance::Instance> {
        self.instance.clone()
    }

    pub fn get_device(&self) -> Arc<device::Device> {
        self.device.clone()
    }

    pub fn get_queue(&self) -> Arc<device::Queue> {
        self.queue.clone()
    }

    //Stuff

    pub fn create_buffer_from_data<T: 'static + Sized>(&self, data: Vec<T>) -> Arc<CpuAccessibleBuffer<Vec<T>>> {
        CpuAccessibleBuffer::from_data(self.get_device(), BufferUsage::all(), data).unwrap()
    }
}