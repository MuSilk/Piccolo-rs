use crate::{core::math::{quaternion::Quaternion, vector3::Vector3}, function::framework::component::{camera_component::CameraMode, component::{Component, ComponentTrait}}};


#[derive(Clone)]
pub struct CharacterComponent {
    m_component: Component,

    pub m_position: Vector3,
    pub m_rotation: Quaternion,

    pub m_rotation_buffer: Quaternion,
    pub m_rotation_dirty: bool,

    pub m_original_camera: CameraMode,
    pub m_is_free_camera: bool,
}

impl ComponentTrait for CharacterComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CharacterComponent {

    pub fn new() -> Self {
        Self {
            m_component: Component::default(),
            m_position: Vector3::new(0.0, 0.0, 0.0),
            m_rotation: Quaternion::identity(),

            m_rotation_buffer: Quaternion::identity(),
            m_rotation_dirty: false,
            m_original_camera: CameraMode::ThirdPerson,
            m_is_free_camera: false,
        }
    }
    pub fn get_position(&self) -> Vector3 {
        self.m_position
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.m_rotation = rotation;
    }

    pub fn get_rotation(&self) -> Quaternion {
        self.m_rotation
    }
}