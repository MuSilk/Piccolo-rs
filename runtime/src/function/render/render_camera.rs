use std::sync::Mutex;

use nalgebra_glm::{vec4, Mat4, Quat, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCameraType{
    Editor,
    Motor,
}

pub struct RenderCamera{
    m_current_type: RenderCameraType,
    m_position: Vec3,
    m_rotation: Quat,
    m_inv_rotation: Quat,
    m_znear: f32,
    m_zfar: f32,
    m_up_axis: Vec3,
    m_view_matrices : Vec<Mat4>,
    m_aspect: f32,
    m_fovx: f32,
    m_fovy: f32,
    m_view_matrix_mutex: Mutex<()>,
}

impl Default for RenderCamera {
    fn default() -> Self {
        Self {
            m_current_type: RenderCameraType::Editor,
            m_position: nalgebra_glm::vec3(0.0, 0.0, 5.0),
            m_rotation: nalgebra_glm::quat_identity(),
            m_inv_rotation: nalgebra_glm::quat_identity(),
            m_znear: 0.1,
            m_zfar: 1000.0,
            m_up_axis: Self::Z,
            m_view_matrices: vec![Mat4::identity()],
            m_aspect: 0.0,
            m_fovx: 89.0,
            m_fovy: 0.0,
            m_view_matrix_mutex: Mutex::new(()),
        }
    }
}

impl RenderCamera {
    const X : Vec3 = Vec3::new(1.0, 0.0, 0.0);
    const Y : Vec3 = Vec3::new(0.0, 1.0, 0.0);
    const Z : Vec3 = Vec3::new(0.0, 0.0, 1.0);
    const MIN_FOV : f32 = 10.0;
    const MAX_FOV : f32 = 89.0;
    const MAIN_VIEW_MATRIX_INDEX : i32 = 0;
}

impl RenderCamera {

    pub fn position(&self) -> &Vec3 {
        &self.m_position
    }

    pub fn rotation(&self) -> &Quat {
        &self.m_rotation
    }

    pub fn forward(&self) -> Vec3 {
        let res = nalgebra_glm::quat_cast(&self.m_inv_rotation) * vec4(Self::Y.x, Self::Y.y, Self::Y.z, 1.0);
        res.xyz()/res.w
    }

    pub fn up(&self) -> Vec3 {
        let res = nalgebra_glm::quat_cast(&self.m_inv_rotation) * vec4(Self::Z.x, Self::Z.y, Self::Z.z, 1.0);
        res.xyz()/res.w
    }

    pub fn right(&self) -> Vec3 {
        let res = nalgebra_glm::quat_cast(&self.m_inv_rotation) * vec4(Self::X.x, Self::X.y, Self::X.z, 1.0);
        res.xyz()/res.w
    }

    pub fn get_fov(&self) -> Vec2 {
        Vec2::new(self.m_fovx, self.m_fovy)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let _guard = self.m_view_matrix_mutex.lock().unwrap();
        match self.m_current_type {
            RenderCameraType::Editor => 
                nalgebra_glm::look_at(&self.position(), &(self.position() + self.forward()), &self.up()),
            RenderCameraType::Motor => 
                self.m_view_matrices[Self::MAIN_VIEW_MATRIX_INDEX as usize]
        }
    }

    pub fn get_pers_proj_matrix(&self) -> Mat4 { 
        let fix_mat = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        fix_mat * nalgebra_glm::perspective(self.m_aspect, self.m_fovy.to_radians(), self.m_znear, self.m_zfar)
    }

    pub fn get_look_at_matrix(&self) -> Mat4 { 
        nalgebra_glm::look_at(&self.m_position, &(self.m_position + self.forward()), &self.up())
    }

    pub fn get_fovy_deprecated(&self) -> f32 {
        self.m_fovy
    }


    pub fn set_current_camera_type(&mut self, camera_type: RenderCameraType) {
        let _guard = self.m_view_matrix_mutex.lock();
        self.m_current_type = camera_type;
    }

    pub fn set_main_view_matrix(&mut self, view_matrix: &Mat4, camera_type: RenderCameraType) {
        let _guard = self.m_view_matrix_mutex.lock().unwrap();
        self.m_current_type = camera_type;
        self.m_view_matrices[Self::MAIN_VIEW_MATRIX_INDEX as usize] = *view_matrix;
        
        let view_matrix_inv = nalgebra_glm::affine_inverse(view_matrix.clone());
        self.m_position = nalgebra_glm::vec3(
            view_matrix_inv[(0, 3)],
            view_matrix_inv[(1, 3)],
            view_matrix_inv[(2, 3)],
        );
    }

    pub fn move_camera(&mut self, delta: &Vec3) {
        self.m_position += delta;
    }

    pub fn rotate_camera(&mut self, delta: &Vec2){
        let mut delta = nalgebra_glm::radians(delta);

        let dot = self.m_up_axis.dot(&self.forward());
        if (dot < -0.99 && delta.x > 0.0) || (dot > 0.99 && delta.x < 0.0) {
            delta.x = 0.0;
        }

        let pitch = nalgebra_glm::quat_angle_axis(delta.x, &Self::X);
        let yaw = nalgebra_glm::quat_angle_axis(delta.y, &Self::Z);
        self.m_rotation = pitch * self.m_rotation * yaw;
        self.m_inv_rotation = self.m_rotation.conjugate();
    }

    pub fn zoom_camera(&mut self, offset: f32) {
        self.m_fovx = (self.m_fovx - offset).clamp(Self::MIN_FOV, Self::MAX_FOV);
    }

    pub fn look_at(&mut self, target: &Vec3, up: &Vec3) {
        let view_matrix = nalgebra_glm::look_at(&self.m_position, target, up);
        let rotation_matrix = nalgebra_glm::mat4_to_mat3(&view_matrix);
        self.m_inv_rotation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
        self.m_rotation = self.m_inv_rotation.conjugate();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.m_aspect = aspect;
        self.m_fovy = (((self.m_fovx * 0.5).to_radians().tan()/self.m_aspect).atan() * 2.0).to_degrees();
    }

    pub fn set_fov_x(&mut self, fovx: f32) {
        self.m_fovx = fovx;
    }
}