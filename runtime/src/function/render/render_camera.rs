use std::sync::Mutex;

use crate::core::math::{self, matrix4::Matrix4x4, quaternion::Quaternion, vector2::Vector2, vector3::Vector3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCameraType{
    Editor,
    Motor,
}

pub struct RenderCamera{
    m_current_type: RenderCameraType,
    m_position: Vector3,
    m_rotation: Quaternion,
    m_inv_rotation: Quaternion,
    m_znear: f32,
    m_zfar: f32,
    m_up_axis: Vector3,
    m_view_matrices : Vec<Matrix4x4>,
    m_aspect: f32,
    m_fovx: f32,
    m_fovy: f32,
    m_view_matrix_mutex: Mutex<()>,
}

impl Default for RenderCamera {
    fn default() -> Self {
        Self {
            m_current_type: RenderCameraType::Editor,
            m_position: Vector3::new(0.0, -1.0, 0.0),
            m_rotation: Quaternion::identity(),
            m_inv_rotation: Quaternion::identity(),
            m_znear: 0.1,
            m_zfar: 1000.0,
            m_up_axis: Self::Z,
            m_view_matrices: vec![Matrix4x4::identity()],
            m_aspect: 0.0,
            m_fovx: 89.0,
            m_fovy: 0.0,
            m_view_matrix_mutex: Mutex::new(()),
        }
    }
}

impl RenderCamera {
    const X : Vector3 = Vector3::new(1.0, 0.0, 0.0);
    const Y : Vector3 = Vector3::new(0.0, 1.0, 0.0);
    const Z : Vector3 = Vector3::new(0.0, 0.0, 1.0);
    const MIN_FOV : f32 = 10.0;
    const MAX_FOV : f32 = 89.0;
    const MAIN_VIEW_MATRIX_INDEX : i32 = 0;
}

impl RenderCamera {

    pub fn position(&self) -> &Vector3 {
        &self.m_position
    }

    pub fn rotation(&self) -> &Quaternion {
        &self.m_rotation
    }

    pub fn forward(&self) -> Vector3 {
        self.m_inv_rotation * Self::Y
    }

    pub fn up(&self) -> Vector3 {
        self.m_inv_rotation * Self::Z
    }

    pub fn right(&self) -> Vector3 {
        self.m_inv_rotation * Self::X
    }

    pub fn get_fov(&self) -> Vector2 {
        Vector2::new(self.m_fovx, self.m_fovy)
    }

    pub fn get_view_matrix(&self) -> Matrix4x4 {
        let _guard = self.m_view_matrix_mutex.lock().unwrap();
        match self.m_current_type {
            RenderCameraType::Editor => 
                math::look_at(&self.position(), &(self.position() + self.forward()), &self.up()),
            RenderCameraType::Motor => 
                self.m_view_matrices[Self::MAIN_VIEW_MATRIX_INDEX as usize]
        }
    }

    pub fn get_pers_proj_matrix(&self) -> Matrix4x4 { 
        let fix_mat = Matrix4x4::from_columns(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        );
        fix_mat * math::perspective(self.m_fovy.to_radians(), self.m_aspect, self.m_znear, self.m_zfar)
    }

    pub fn get_look_at_matrix(&self) -> Matrix4x4 { 
        math::look_at(&self.m_position, &(self.m_position + self.forward()), &self.up())
    }

    pub fn get_fovy_deprecated(&self) -> f32 {
        self.m_fovy
    }


    pub fn set_current_camera_type(&mut self, camera_type: RenderCameraType) {
        let _guard = self.m_view_matrix_mutex.lock();
        self.m_current_type = camera_type;
    }

    pub fn set_main_view_matrix(&mut self, view_matrix: Matrix4x4, camera_type: RenderCameraType) {
        let _guard = self.m_view_matrix_mutex.lock().unwrap();
        self.m_current_type = camera_type;
        self.m_view_matrices[Self::MAIN_VIEW_MATRIX_INDEX as usize] = view_matrix;

        let s = Vector3::new(view_matrix[0][0], view_matrix[1][0], view_matrix[2][0]);
        let u = Vector3::new(view_matrix[0][1], view_matrix[1][1], view_matrix[2][1]);
        let f = Vector3::new(view_matrix[0][2], -view_matrix[1][2], -view_matrix[2][2]);
        self.m_position = s *(-view_matrix[3][0]) + u * (-view_matrix[3][1]) + f * view_matrix[3][2];
    }

    pub fn move_camera(&mut self, delta: &Vector3) {
        self.m_position += delta;
    }

    pub fn rotate_camera(&mut self, delta: &Vector2){
        let mut delta = Vector2::new(delta.x.to_radians(), delta.y.to_radians());

        let dot = self.m_up_axis.dot(&self.forward());
        if (dot < -0.99 && delta.x > 0.0) || (dot > 0.99 && delta.x < 0.0) {
            delta.x = 0.0;
        }

        let pitch = Quaternion::from_angle_axis(delta.x, &Self::X);
        let yaw = Quaternion::from_angle_axis(delta.y, &Self::Z);
        self.m_rotation = pitch * self.m_rotation * yaw;
        self.m_inv_rotation = self.m_rotation.conjugate();
    }

    pub fn zoom_camera(&mut self, offset: f32) {
        self.m_fovx = (self.m_fovx - offset).clamp(Self::MIN_FOV, Self::MAX_FOV);
    }

    pub fn look_at(&mut self, position: Vector3, target: &Vector3, up: &Vector3) {
        self.m_position = position;
        let forward = (target - position).normalize();
        self.m_rotation = forward.get_rotation_to(&Self::Y);

        let right = forward.cross(&up.normalize()).normalize();
        let orth_up = right.cross(&forward);

        let up_rotation = (self.m_rotation * orth_up).get_rotation_to(&Self::Z);

        self.m_rotation = up_rotation * self.m_rotation;

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