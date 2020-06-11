pub mod core;
pub mod textures;
pub mod shapes;
pub mod camera;


extern crate nalgebra_glm as glm;
use std::sync::Arc;

use std::collections::HashMap;

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{AcquireError};
use vulkano::swapchain::{
    ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain,
};
use vulkano::sync;
use vulkano::sync::FlushError;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;


use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};



use crate::core::{ MamgaGfx, Renderer };
use crate::camera::{Camera, get_projection_matrix};


pub struct Window {
    pub event_loop: EventLoop<()>
}


impl MamgaGfx {
    pub fn run<F>(mut self, window: Window, mut update: F)
    where
        F: 'static + FnMut(&mut Renderer) -> (),
    {
        let physical = PhysicalDevice::enumerate(&self.instance)
            .next()
            .expect("no device available");

        let mut framebuffers =
            window_size_dependent_setup(&self.images, self.renderer.render_pass.clone(), &mut self.renderer.dynamic_state);
        let mut recreate_swapchain = false;
        let mut previous_frame_end =
            Some(Box::new(sync::now(self.device.clone())) as Box<dyn GpuFuture>);
            

        window.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    recreate_swapchain = true;
                }
                Event::RedrawEventsCleared => {
                    previous_frame_end.as_mut().unwrap().cleanup_finished();
                    
                    if recreate_swapchain {
                        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
                        let (new_swapchain, new_images) = match self.swapchain
                            .recreate_with_dimensions(dimensions)
                        {
                            Ok(r) => r,
                            Err(
                                vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions,
                            ) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                        self.swapchain = new_swapchain;
                        framebuffers = window_size_dependent_setup(
                            &new_images,
                            self.renderer.render_pass.clone(),
                            &mut self.renderer.dynamic_state,
                        );
                        self.images = new_images;
                        recreate_swapchain = false;
                    }

                    let (image_num, suboptimal, acquire_future) =
                        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };
                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    self.renderer.command_buffer_builder = Option::from(AutoCommandBufferBuilder::primary_one_time_submit(
                        self.device.clone(),
                        self.queue.family(),
                    )
                    .unwrap()
                    .begin_render_pass(
                        framebuffers[image_num].clone(),
                        false,
                        vec![self.clear_color.into()],
                    )
                    .unwrap());

                    update(&mut self.renderer);

                    let command_buffer = self.renderer.command_buffer_builder.take().unwrap().end_render_pass()
                        .unwrap()
                        .build().unwrap();
                    

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            previous_frame_end = Some(Box::new(future) as Box<_>);
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end =
                                Some(Box::new(sync::now(self.device.clone())) as Box<_>);
                        }
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                            previous_frame_end =
                                Some(Box::new(sync::now(self.device.clone())) as Box<_>);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

pub fn init_renderer() -> (MamgaGfx, Window) {
    let required_extensions = vulkano_win::required_extensions();
    let instance =
        Instance::new(None, &required_extensions, None).expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        },
        [(queue_family, 0.5)].iter().cloned(),
    )
    .expect("failed to create device");
    let queue = queues.next().unwrap();

    use winit::dpi::LogicalSize;
    let _window = WindowBuilder::new().with_inner_size(LogicalSize::new(1200, 800));


    

    let event_loop = EventLoop::new();
    let surface = _window
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .unwrap()
    };
    
    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    let dimensions = images[0].dimensions();

    let mut dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: -1.0..1.0,
        }]),
        ..DynamicState::none()
    };

    let renderer = Renderer {
        dynamic_state,
        device: device.clone(),
        pipelines: HashMap::new(),
        render_pass,
        command_buffer_builder: None,
        camera: Camera::default()
    };

    (
        MamgaGfx {
            queue,
            device,
            instance,
            clear_color: [0.1, 0.1, 0.1, 1.0],
            swapchain,
            images,
            surface,
            renderer
        },
        Window {
            event_loop,
        },
    )
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<winit::window::Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: -1.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
