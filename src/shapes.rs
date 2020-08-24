
use crate::core::{Renderer, Color, Vertex2DColor};
use crate::camera::get_projection_matrix;
use std::sync::Arc;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::framebuffer::Subpass;
use vulkano::buffer::{ CpuAccessibleBuffer, BufferUsage };
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSetImg, PersistentDescriptorSet, PersistentDescriptorSetSampler, FixedSizeDescriptorSetsPool};
use glam::{mat4, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4};

pub struct Rectangle {
    pub position: Vec2,
    pub width: i32,
    pub height: i32
}

static mut ran: bool = false;
impl Renderer {
    pub fn rectangle(&mut self, rectangle: &Rectangle, color: Color) {
        if !self.pipelines.contains_key("rect") {
            init_rect(self);
        }
        let format_color = color.normalise();
        let vertex_buffer = CpuAccessibleBuffer::<[Vertex2DColor]>::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex2DColor {
                    position: [-0.5, -0.5],
                    color: format_color,
                },
                Vertex2DColor {
                    position: [-0.5, 0.5],
                    color: format_color,
                },
                Vertex2DColor {
                    position: [0.5, -0.5],
                    color: format_color,
                },
                Vertex2DColor {
                    position: [0.5, 0.5],
                    color: format_color,
                },
            ]
            .iter()
            .cloned(),
        ).unwrap();



        let ubuf = CpuAccessibleBuffer::from_data(self.device.clone(), BufferUsage::uniform_buffer(), true, {
            // note: this teapot was meant for OpenGL where the origin is at the lower left
            //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
            let dimensions: [f32; 2] = self.dynamic_state.viewports.as_ref().unwrap().get(0).unwrap().dimensions;
            //  let aspect_ratio = dimensions[0] / dimensions[1];
            
            let mvp = get_projection_matrix(Vec2::new(rectangle.width as f32, rectangle.height as f32), rectangle.position, dimensions);
            rec_vs::ty::Data {
               mvp: [mvp.x_axis().into(), mvp.y_axis().into(), mvp.z_axis().into(), mvp.w_axis().into()],
            }
        }).unwrap();
        let pipeline = self.pipelines.get("rect").unwrap();
        let p = pipeline.clone();
        let layout = p.descriptor_set_layout(0).unwrap();
        let set = Arc::new(PersistentDescriptorSet::start(layout.clone())
            .add_buffer(ubuf).unwrap()
            .build().unwrap()
        );
        
        let cmb = self.command_buffer_builder.take().unwrap();
        let res = cmb.draw(pipeline.clone(), &self.dynamic_state, vec!(vertex_buffer), set, ());
        self.command_buffer_builder = Some(res.unwrap());
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

            layout(set = 0, binding = 0) uniform Data {
                uniform mat4 mvp;
            } uniforms;

            out gl_PerVertex {
                vec4 gl_Position;
            };


            void main() {
                gl_Position = uniforms.mvp * vec4(position.xy, 0.0, 1.0);
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
    println!("fires square");
    let pipeline = GraphicsPipeline::start()
    // Defines what kind of vertex input is expected.
    .vertex_input_single_buffer::<crate::core::Vertex2DColor>()
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

    println!("fires square");
    
    draw.pipelines.insert("rect".to_string(), Arc::new(pipeline));
}

