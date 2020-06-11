use nalgebra_glm::{Mat4, Vec2, Vec3};


pub fn get_projection_matrix(width: f32, height: f32, position: Vec2, dimensions: [f32; 2]) -> Mat4 {
    //let projection = nalgebra_glm::ortho(0.0, 1200., 800., 0.0, -1.0, 100.0);
    let proj = nalgebra_glm::ortho_zo(0.0, 1200., 800., 0.0, -1.0, 100.0);
    let camera_pos = nalgebra_glm::vec3(0.0, 0.0, 3.0);
    let camera_front = nalgebra_glm::vec3(0.0, 0.0, -1.0);
    let view = nalgebra_glm::look_at_rh(
        &camera_pos, // Camera is at (4,3,3), in World Space
        &(camera_pos + camera_front), // and looks at the origin
        &nalgebra_glm::Vec3::new(0.0,1.0,0.0)  // Head is up (set to 0,-1,0 to look upside-down)
    );
    let mut model = Mat4::identity();
    let pos = Vec3::new(position.x + (width / 2.), position.y + (height / 2.), 0.);
    model =
        nalgebra_glm::translate(&model, &pos);

    model = nalgebra_glm::translate(
        &model,
        &nalgebra_glm::vec3(0.5 * width, 0.5 * height, 0.0),
    );
    model = nalgebra_glm::rotate(&model, 0.0, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
    model = nalgebra_glm::translate(
        &model,
        &nalgebra_glm::vec3(-0.5 * width, -0.5 * height, 0.0),
    );

    model = nalgebra_glm::scale(&model, &nalgebra_glm::vec3(width, height, 1.0));

    proj * model * view
}



pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
}

impl Camera {

    pub fn get_view(&self) -> Mat4 {
        nalgebra_glm::look_at(&self.position, &self.target, &self.up)
    }

    pub fn default() -> Self {
        
        let position = Vec3::new(0.0,0.0, 3.0);
        let target = Vec3::new(0.0, 0.0, 0.0);
        let direction = nalgebra_glm::normalize(&(position - target));
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&nalgebra_glm::vec3(0.0, 1.0, 0.0), &direction));
        let up = nalgebra_glm::cross(&direction, &right);
        
        Camera {
            position,
            up,
            target
        }
    }
}