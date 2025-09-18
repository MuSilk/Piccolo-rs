use nalgebra_glm::{Mat4, Vec3};

pub enum CameraMovement {
	FORWARD, BACKWARD,
	LEFT, RIGHT,
	UP, DOWN
}

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVITY: f32 = 0.005;
const ZOOM: f32 = 45.0;

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    mouse_sensitivity: f32,
    zoom: f32,

    near: f32, far: f32,
}


impl Camera {
    pub fn new(position: &Vec3) -> Self {
        let mut camera = Self {
            position: position.clone(),
            front: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 0.0, 0.0),
            right: Vec3::new(0.0, 0.0, 0.0),
            world_up: Vec3::new(0.0, 1.0, 0.0),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM,
            near: 0.1,
            far: 100.0,
        };
        camera.update_camera_vectors();
        camera
    }

    pub fn set_target(&mut self, target: &Vec3){
        self.front = (target - self.position).normalize();
        self.right = nalgebra_glm::cross(&self.front, &self.world_up).normalize();
        self.up = nalgebra_glm::cross(&self.right, &self.front).normalize();
        
        self.yaw = self.front.x.atan2(self.front.z);
        self.pitch = self.front.y.asin();
        println!("{:?}",self);
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let target = self.position + self.front;
        nalgebra_glm::look_at_rh(&self.position, &target, &self.up)
    }

    pub fn get_projection_matrix(&self, width: u32, height: u32) -> Mat4 {
        let correction = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0,-1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0
        );
        correction * nalgebra_glm::perspective(
            width as f32 / height as f32,
            self.zoom.to_radians(),   
            self.near, self.far
        )
    }
    pub fn process_keyboard(&mut self, direction:&CameraMovement, delta_time:f32) {
		let velocity = self.movement_speed * delta_time;
        match direction {
            CameraMovement::FORWARD => self.position += self.front * velocity,
            CameraMovement::BACKWARD => self.position -= self.front * velocity,
            CameraMovement::LEFT => self.position -= self.right * velocity,
            CameraMovement::RIGHT => self.position += self.right * velocity,
            CameraMovement::UP => self.position += self.up * velocity,
            CameraMovement::DOWN => self.position -= self.up * velocity,
        }
	}

    pub fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, constrain_pitch: bool) {
        let xoffset = xoffset * self.mouse_sensitivity;
        let yoffset = yoffset * self.mouse_sensitivity;

		self.yaw += xoffset;
		self.pitch += yoffset;

		if constrain_pitch{
            self.pitch = self.pitch.clamp(-89.0, 89.0);
		}

		self.update_camera_vectors();

	}
    fn update_camera_vectors(&mut self) {
        let front = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        self.front = front.normalize();
        self.right = nalgebra_glm::cross(&self.front, &self.world_up).normalize();
        self.up = nalgebra_glm::cross(&self.right, &self.front).normalize();
    }

}