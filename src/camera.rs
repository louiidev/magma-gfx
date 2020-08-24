// use nalgebra_glm::{Mat4, Vec2, Vec3};
use glam::{mat4, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4};


pub fn ortho_matrix_vulk(left: f32, right: f32 , bottom: f32, top: f32, z_near: f32, z_far: f32 ) -> Mat4 {
    let a = 2.0 / (right - left);
    let b = 2.0 / (bottom - top);
    let c = 1. / (z_near - z_far);

    let tx = -(right + left) / (right - left);
    let ty = -(bottom + top) / (bottom - top);
    let tz = z_near / (z_near - z_far);

    Mat4::from_cols(
        Vec4::new(a, 0.0, 0.0, 0.),
        Vec4::new(0.0, b, 0.0, 0.),
        Vec4::new(0.0, 0.0, c, 0.),
        Vec4::new(tx, ty, tz, 1.0),
    )
}

pub fn get_projection_matrix(size: Vec2, position: Vec2, dimensions: [f32; 2]) -> Mat4 {
    let proj = ortho_matrix_vulk(0.0, *dimensions.get(0).unwrap(), *dimensions.get(1).unwrap(), 0.0, -1.0, 1.0);
    let camera_pos = vec3(0.0, 0.0, 3.0);
    let camera_front = vec3(0.0, 0.0, -1.0);
    let view = Mat4::look_at_lh(
        camera_pos, // Camera is at (4,3,3), in World Space
        camera_pos + camera_front, // and looks at the origin
        Vec3::new(0.0,-1.0,0.0)  // Head is up (set to 0,-1,0 to look upside-down)
    );
    let model = Mat4::from_scale_rotation_translation(Vec3::new(size.x(), size.y(), 1.0), Quat::identity(), Vec3::new(position.x(), position.y(), 1.0));

    let mut correction = Mat4::identity();
    
    proj * model
}



pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
}

impl Camera {

    pub fn get_view(&self) -> Mat4 {
        Mat4::look_at_lh(self.position, self.target, self.up)
    }

    pub fn default() -> Self {
        
        let position = Vec3::new(0.0,0.0, 3.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::normalize(position - target);
        let right = Vec3::normalize(Vec3::cross(Vec3::new(0.0, 1.0, 0.0), direction));
        let up = Vec3::cross(direction, right);
        
        Camera {
            position,
            up,
            target
        }
    }
}