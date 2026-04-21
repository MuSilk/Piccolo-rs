use std::any::Any;

use crate::{
    core::math::{
        matrix4::Matrix4x4, quaternion::Quaternion, transform::Transform, vector3::Vector3,
    },
    engine::Engine,
    function::framework::{component::component::ComponentTrait, object::object::GObject},
};

#[derive(Clone, Debug)]
pub struct TransformComponent {
    m_transform: Transform,
    m_transform_buffer: [Transform; 2],
    m_current_index: usize,
    m_next_index: usize,
    m_is_dirty: bool,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            m_transform: Transform::default(),
            m_transform_buffer: [Transform::default(), Transform::default()],
            m_current_index: 0,
            m_next_index: 1,
            m_is_dirty: false,
        }
    }
}

impl TransformComponent {
    pub fn post_load_resource(&mut self, transform: Transform) {
        self.m_transform = transform;
        self.m_transform_buffer[0] = self.m_transform.clone();
        self.m_transform_buffer[1] = self.m_transform.clone();
        self.m_is_dirty = true;
    }
    pub fn get_matrix(&self) -> Matrix4x4 {
        self.m_transform_buffer[self.m_current_index].get_matrix()
    }

    pub fn set_position(&mut self, position: Vector3) {
        self.m_transform_buffer[self.m_next_index].set_position(position);
        self.m_transform.set_position(position);
        self.m_is_dirty = true;
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.m_transform_buffer[self.m_next_index].set_rotation(rotation);
        self.m_transform.set_rotation(rotation);
        self.m_is_dirty = true;
    }

    pub fn get_position(&self) -> &Vector3 {
        self.m_transform_buffer[self.m_current_index].get_position()
    }

    pub fn get_rotation(&self) -> &Quaternion {
        self.m_transform_buffer[self.m_current_index].get_rotation()
    }

    pub fn tick(&mut self, engine: &Engine) {
        (self.m_current_index, self.m_next_index) = (self.m_next_index, self.m_current_index);

        if engine.is_editor_mode() {
            self.m_transform_buffer[self.m_next_index] = self.m_transform.clone();
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.m_is_dirty
    }

    pub fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.m_is_dirty = is_dirty;
    }
}

impl ComponentTrait for TransformComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn tick(&mut self, engine: &Engine, _gobject: &GObject, _delta_time: f32) {
        self.tick(engine);
    }
}
