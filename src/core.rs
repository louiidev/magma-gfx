use vulkano::device::Queue;
use vulkano::device::{Device, DeviceExtensions, Features};
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::{
    ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Surface, Swapchain,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};

use crate::camera::Camera;

use std::collections::HashMap;

pub struct MamgaGfx {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub instance: Arc<Instance>,
    pub clear_color: [f32; 4],
    pub renderer: Renderer,
    pub swapchain: Arc<Swapchain<winit::window::Window>>,
    pub images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
    pub surface: Arc<Surface<winit::window::Window>>
}

pub struct Renderer {
    pub device: Arc<Device>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub pipelines: HashMap<String, Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    pub command_buffer_builder: Option<AutoCommandBufferBuilder>,
    pub dynamic_state: DynamicState,
    pub camera: Camera
}


pub enum RenderTypes {
    Texture2D,
    Rectangle
}

impl MamgaGfx {
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color.normalise();
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex2D {
    pub position: [f32; 2]
}
impl Vertex2D {
    pub fn new(position: [f32; 2]) -> Self {
        Vertex2D { position }
    }
}

vulkano::impl_vertex!(Vertex2D, position);

#[derive(Default, Debug, Clone)]
pub struct Vertex2DColor {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
impl Vertex2DColor {
    pub fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Vertex2DColor { position, color }
    }
}

vulkano::impl_vertex!(Vertex2DColor, position, color);

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