use crate::core::{Renderer, MamgaGfx, Color, Vertex2D};
use crate::camera::get_projection_matrix;
use std::sync::Arc;
use vulkano::pipeline::{
    GraphicsPipeline,
    GraphicsPipelineAbstract
};


use vulkano::framebuffer::Subpass;
use vulkano::buffer::{ CpuAccessibleBuffer, BufferUsage };
use vulkano::image::ImmutableImage;
use vulkano::image::Dimensions;
use vulkano::format::R8G8B8A8Unorm;
use image::GenericImageView;
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSetImg, PersistentDescriptorSet, PersistentDescriptorSetSampler, FixedSizeDescriptorSetsPool};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::buffer::cpu_pool::CpuBufferPool;

use glam::{mat4, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4};

pub struct Texture2D {
    pub width: i32,
    pub height: i32,
    ubuf: CpuBufferPool<texture_vs::ty::Data>,
    pool: FixedSizeDescriptorSetsPool,
    sampler: Arc<Sampler>,
    image: Arc<ImmutableImage<R8G8B8A8Unorm>>
}


impl Texture2D {
    pub fn load(gfx: &mut MamgaGfx, path: String) -> Texture2D {
        
        let vs = texture_vs::Shader::load(gfx.device.clone()).unwrap();
        let fs = texture_fs::Shader::load(gfx.device.clone()).unwrap();
        
        let pipeline = GraphicsPipeline::start()
        // Defines what kind of vertex input is expected.
        .vertex_input_single_buffer::<crate::core::Vertex2D>()
        // The vertex shader.
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_strip()
        // Defines the viewport (explanations below).
        .viewports_dynamic_scissors_irrelevant(1)
        // The fragment shader.
        .fragment_shader(fs.main_entry_point(), ())
        .blend_alpha_blending()
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(gfx.renderer.render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(gfx.renderer.device.clone())
        .unwrap();
        let ubuf = CpuBufferPool::new(gfx.device.clone(), BufferUsage::uniform_buffer());
        
        let loaded_image = image::open(path).unwrap();
        let dimensions = loaded_image.dimensions();
        let image = loaded_image.to_rgba().to_vec();
        
        let (texture, _tex_future) = {
            ImmutableImage::from_iter(
                image.iter().cloned(),
                Dimensions::Dim2d { width: dimensions.0, height: dimensions.1 },
                R8G8B8A8Unorm,
                gfx.queue.clone()
            ).unwrap()
        };

        //let uniform_buffer = CpuBufferPool::<texture_vs::ty::Data>::new(gfx.device.clone(), BufferUsage::all());

        let sampler = Sampler::new(gfx.device.clone(), Filter::Nearest, Filter::Nearest,
        MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();
        
        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();
        let pool = FixedSizeDescriptorSetsPool::new(layout.clone());

        gfx.renderer.pipelines.insert("texture".to_string(), Arc::new(pipeline));
       
        Texture2D {
            image: texture,
            ubuf,
            sampler,
            pool,
            width: dimensions.0 as i32,
            height: dimensions.1 as i32
        }
    }
}


static mut ran: bool = false;


impl Renderer {
    pub fn texture(&mut self, texture: &mut Texture2D, position: Vec2) {
        self.texture_pro(texture, position, 1.0);
    }
    pub fn texture_pro(&mut self, texture: &mut Texture2D, position: Vec2, scale: f32) {
        let vertex_buffer = CpuAccessibleBuffer::<[Vertex2D]>::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex2D {
                    position: [-0.5, -0.5],
                },
                Vertex2D {
                    position: [-0.5, 0.5],
                },
                Vertex2D {
                    position: [0.5, -0.5],
                },
                Vertex2D {
                    position: [0.5, 0.5],
                },
            ]
            .iter()
            .cloned()
    ).unwrap();

    let uniform_buffer_subbuffer = {
        // note: this teapot was meant for OpenGL where the origin is at the lower left
        //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
        let dimensions: [f32; 2] = self.dynamic_state.viewports.as_ref().unwrap().get(0).unwrap().dimensions;
        let projection = get_projection_matrix(Vec2::new(texture.width as f32 * scale, texture.height as f32 * scale), position, dimensions);
        
        let uniform_data = texture_vs::ty::Data {
            projection: [projection.x_axis().into(), projection.y_axis().into(), projection.z_axis().into(), projection.w_axis().into()]
        };
        
        texture.ubuf.next(uniform_data).unwrap()
    };

    let set = texture.pool.next()
    .add_buffer(uniform_buffer_subbuffer).unwrap()
    .add_sampled_image(texture.image.clone(), texture.sampler.clone()).unwrap()
    
    .build().unwrap();
    let cmb = self.command_buffer_builder.take().unwrap();
    let res = cmb.draw(self.pipelines.get("texture").unwrap().clone(), &self.dynamic_state, vec!(vertex_buffer), set, ());
    self.command_buffer_builder = Some(res.unwrap());
}

}


pub mod texture_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec2 tex_coords;

            layout(set = 0, binding = 0) uniform Data {
                uniform mat4 projection;
            } uniforms;

            void main() {
                gl_Position = uniforms.projection * vec4(position, 0.0, 1.0);
                tex_coords = position + vec2(0.5);
            }
        "
    }
}

mod texture_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 tex_coords;
            layout(location = 0) out vec4 f_color;
            layout(set = 0, binding = 1) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, tex_coords);
            }
        "
    }
}