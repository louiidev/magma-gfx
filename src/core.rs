use vulkano::device::Queue;
use vulkano::device::{Device, DeviceExtensions, Features};
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::framebuffer::RenderPassAbstract;

use std::collections::HashMap;

pub struct MamgaGfx {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub instance: Arc<Instance>,
    pub clear_color: [f32; 4],
}

pub struct Renderer {
    pub device: Arc<Device>,
    pub render_queue: Vec<RenderItem>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub pipelines: HashMap<String, Arc<dyn GraphicsPipelineAbstract + Send + Sync>>
}

pub struct RenderItem {
    pub ty: String,
    pub vertex_buffer: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Vertex]>>
}

impl MamgaGfx {
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color.normalise();
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
impl Vertex {
    pub fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Vertex { position, color }
    }
}

vulkano::impl_vertex!(Vertex, position, color);

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

impl Color {
    pub fn normalise(&self) -> [f32; 4] {
        [self.r as f32 / 255., self.g as f32 / 255., self.b as f32 / 255., self.a as f32 / 255.]
    }
    pub fn new(r: u8, g: u8, b: u8) -> Self {
            Color {
                r,
                g,
                b,
                a: 255
            }
        }

        pub fn new_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
                Color {
                    r,
                    g,
                    b,
                    a
                }
            }
}