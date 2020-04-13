use cgmath::Vector2;
use crate::core::{Renderer, Color, RenderItem, Vertex};
use std::sync::Arc;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::framebuffer::Subpass;
use vulkano::buffer::{ CpuAccessibleBuffer, BufferUsage };

pub struct Rectangle {
    pub position: Vector2<f32>,
    pub width: i32,
    pub height: i32
}

impl Renderer {
    pub fn rectangle(&mut self, rectangle: &Rectangle, color: Color) {
        if !self.pipelines.contains_key("rect") {
            init_rect(self);
        }
        let format_color = color.normalise();
        self.render_queue.push(RenderItem {
            ty: "rect".to_string(),
            vertex_buffer: CpuAccessibleBuffer::<[Vertex]>::from_iter(
                self.device.clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, -0.5],
                        color: format_color,
                    },
                    Vertex {
                        position: [-0.5, 0.5],
                        color: format_color,
                    },
                    Vertex {
                        position: [0.5, -0.5],
                        color: format_color,
                    },
                    Vertex {
                        position: [0.5, 0.5],
                        color: format_color,
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        })
    }
}

mod rec_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            #extension GL_ARB_separate_shader_objects : enable

            // NOTE: names must match the `Vertex` struct in Rust
            layout(location = 0) in vec2 position;
            layout(location = 1) in vec4 color;

            layout(location = 0) out vec4 fragColor;

            out gl_PerVertex {
                vec4 gl_Position;
            };

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                fragColor = color;
            }
        "
    }
}

mod rec_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) in vec4 fragColor;
            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = fragColor;
            }
        "
    }
}


pub fn init_rect(draw: &mut Renderer) {
    let vs = rec_vs::Shader::load(draw.device.clone()).unwrap();
    let fs = rec_fs::Shader::load(draw.device.clone()).unwrap();
    let pipeline = GraphicsPipeline::start()
    // Defines what kind of vertex input is expected.
    .vertex_input_single_buffer::<crate::core::Vertex>()
    // The vertex shader.
    .vertex_shader(vs.main_entry_point(), ())
    .triangle_strip()
    // Defines the viewport (explanations below).
    .viewports_dynamic_scissors_irrelevant(1)
    // The fragment shader.
    .fragment_shader(fs.main_entry_point(), ())
    .blend_alpha_blending()
    // This graphics pipeline object concerns the first pass of the render pass.
    .render_pass(Subpass::from(draw.render_pass.clone(), 0).unwrap())
    // Now that everything is specified, we call `build`.
    .build(draw.device.clone())
    .unwrap();

    pipeline = pipeline.vertex_shader().build();

    
    
    draw.pipelines.insert("rect".to_string(), Arc::new(pipeline));
}

