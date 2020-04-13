pub mod core;
pub mod textures;
pub mod shapes;

use std::sync::Arc;

use std::collections::HashMap;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::CommandBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Queue;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::swapchain::{AcquireError, SwapchainCreationError};
use vulkano::swapchain::{
    ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain,
};
use vulkano::sync;
use vulkano::sync::FlushError;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;

use spin_sleep::LoopHelper;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::core::{ Vertex, MamgaGfx, Renderer };

const default_clear: [f32; 4] = [0.1, 0.1, 0.1, 1.0];


impl MamgaGfx {
    pub fn run<F>(mut self, window: winit::window::WindowBuilder, mut update: F)
    where
        F: 'static + FnMut(&mut Renderer) -> (),
    {
        let events_loop = EventLoop::new();
        let surface = window
            .build_vk_surface(&events_loop, self.instance.clone())
            .unwrap();
        let physical = PhysicalDevice::enumerate(&self.instance)
            .next()
            .expect("no device available");
        let (mut swapchain, images) = {
            let caps = surface.capabilities(physical).unwrap();
            let usage = caps.supported_usage_flags;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = surface.window().inner_size().into();

            Swapchain::new(
                self.device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                usage,
                &self.queue,
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
            vulkano::single_pass_renderpass!(self.device.clone(),
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

        let mut dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [1024.0, 1024.0],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };

        let mut framebuffers =
            window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
        let mut recreate_swapchain = false;
        let mut previous_frame_end =
            Some(Box::new(sync::now(self.device.clone())) as Box<dyn GpuFuture>);

        let mut renderer = Renderer {
            render_pass: render_pass.clone(),
            device: self.device.clone(),
            pipelines: HashMap::new(),
            render_queue: Vec::new()
        };

        events_loop.run(move |event, _, control_flow| {
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
                    update(&mut renderer);
                    if recreate_swapchain {
                        let dimensions: [u32; 2] = surface.window().inner_size().into();
                        let (new_swapchain, new_images) = match swapchain
                            .recreate_with_dimensions(dimensions)
                        {
                            Ok(r) => r,
                            // This error tends to happen when the user is manually resizing the window.
                            // Simply restarting the loop is the easiest way to fix this issue.
                            Err(
                                vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions,
                            ) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                        swapchain = new_swapchain;
                        // Because framebuffers contains an Arc on the old swapchain, we need to
                        // recreate framebuffers as well.
                        framebuffers = window_size_dependent_setup(
                            &new_images,
                            render_pass.clone(),
                            &mut dynamic_state,
                        );
                        recreate_swapchain = false;
                    }

                    let (image_num, suboptimal, acquire_future) =
                        match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
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

                    let mut command_builder = AutoCommandBufferBuilder::primary_one_time_submit(
                        self.device.clone(),
                        self.queue.family(),
                    )
                    .unwrap()
                    .begin_render_pass(
                        framebuffers[image_num].clone(),
                        false,
                        vec![self.clear_color.into()],
                    )
                    .unwrap();

                    for queue in &renderer.render_queue {
                        command_builder = command_builder.draw(
                            renderer.pipelines.get(&queue.ty).unwrap().clone(),
                            &dynamic_state,
                            vec!(queue.vertex_buffer.clone()),
                            (),
                            (),
                        )
                        .unwrap()
                        
                    }

                    let command_buffer = command_builder.end_render_pass()
                        .unwrap()
                        .build()
                        .unwrap();
                    

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(self.queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(self.queue.clone(), swapchain.clone(), image_num)
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

pub fn init_renderer() -> (MamgaGfx, winit::window::WindowBuilder) {
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
    let window = WindowBuilder::new();

    (
        MamgaGfx {
            queue,
            device,
            instance,
            clear_color: default_clear
        },
        window,
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
        depth_range: 0.0..1.0,
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
