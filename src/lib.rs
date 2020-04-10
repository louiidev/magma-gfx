pub mod textures;

use std::sync::Arc;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::instance::{ Instance, InstanceExtensions, PhysicalDevice};
use vulkano::command_buffer::CommandBuffer;
use vulkano::sync::GpuFuture;
use vulkano::device::Queue;

#[derive(Debug)]
pub struct MamgaGfx {
    device: Arc<Device>,
    queue: Arc<Queue>,
    instance: Arc<Instance>
}


pub fn init_renderer() -> MamgaGfx {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    let queue_family = physical
    .queue_families()
    .find(|&q| q.supports_graphics())
    .expect("couldn't find a graphical queue family");

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .expect("failed to create device");
    let queue = queues.next().unwrap();

    MamgaGfx {
        queue,
        device,
        instance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
