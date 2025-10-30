use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{core::math::{matrix4::Matrix4x4, transform::Transform, vector3::Vector3}, engine::G_IS_EDITOR_MODE, function::framework::{component::component::{Component, ComponentTrait}, level::level::Level, object::object_id_allocator::GObjectID}};


#[derive(Clone, Serialize, Deserialize)]
pub struct TransformComponent {
    #[serde(skip)]
    m_component: Component,
    m_transform: Transform,
    #[serde(skip)]
    m_transform_buffer: [Transform; 2],
    #[serde(skip)]
    m_current_index: usize,
    #[serde(skip)]
    m_next_index: usize,
}

impl Default for  TransformComponent {
    fn default() -> Self {
        Self {
            m_component: Component::default(),
            m_transform: Transform::default(),
            m_transform_buffer: [Transform::default(), Transform::default()],
            m_current_index: 0,
            m_next_index: 1,
        }
    }
}

#[typetag::serde]
impl ComponentTrait for TransformComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }

    fn post_load_resource(&mut self, _parent_level: &Rc<RefCell<Level>>, parent_object: GObjectID) {
        self.m_component.m_parent_object = parent_object;
        self.m_transform_buffer[0] = self.m_transform.clone();
        self.m_transform_buffer[1] = self.m_transform.clone();
        self.m_component.m_is_dirty = true;
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_box(&self) -> Box<dyn ComponentTrait> {
        Box::new(self.clone())
    }

    fn tick(&mut self, _delta_time: f32) {
        (self.m_current_index, self.m_next_index) = (self.m_next_index, self.m_current_index);

        if unsafe { G_IS_EDITOR_MODE} {
            self.m_transform_buffer[self.m_next_index] = self.m_transform.clone();
        }
    }
}

impl TransformComponent {
    pub fn get_matrix(&self) -> Matrix4x4 {
        self.m_transform_buffer[self.m_current_index].get_matrix()
    }

    pub fn set_position(&mut self, position: Vector3) {
        self.m_transform_buffer[self.m_next_index].set_position(position);
        self.m_transform.set_position(position);
        self.m_component.m_is_dirty = true;
    }
}