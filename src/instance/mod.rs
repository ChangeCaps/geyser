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

            Arc::new(ComputePipeline::new($instance.get_device(), 
                     &cs::Shader::load($instance.get_device()).unwrap().main_entry_point(), 
                     &()).unwrap())
        }
    };
}

#[macro_export]
macro_rules! create_descriptor_set {
    ([$($buffer:expr)*], $pipeline:expr) => {
        {
            use std::sync::Arc;

            let mut set = vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start($pipeline.clone(), 0);

            Arc::new(set$(.add_buffer($buffer.clone()).unwrap())+.build().unwrap())
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

    ///Get's a clone of the ['vulkano::instance::Instance']
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

    pub fn create_buffer_from_data<T: 'static + Sized>(&self, data: Vec<T>) -> Arc<CpuAccessibleBuffer<[T]>> {
        CpuAccessibleBuffer::from_iter(self.get_device(), BufferUsage::all(), data.into_iter()).unwrap()
    }
    
    pub fn dispatch<L: 'static, R: 'static, C: 'static>(&self, size: [u32; 3], pipeline: Arc<ComputePipeline<C>>, 
        set: Arc<descriptor_set::PersistentDescriptorSet<L, R>>)
        
        where L: Send + Sync,
              R: Send + Sync,
              C: Send + Sync,
              ComputePipeline<C>: ComputePipelineAbstract,
              descriptor_set::PersistentDescriptorSet<L, R>: DescriptorSet,
    {
        use vulkano::command_buffer::CommandBuffer;
        use vulkano::sync::GpuFuture;

        let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::new(
            self.get_device(), self.get_queue().family()).unwrap()
                .dispatch(size, pipeline.clone(), set.clone(), ()).unwrap()
                .build().unwrap();

        let finished = command_buffer.execute(self.get_queue()).unwrap();

        finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    }
}
