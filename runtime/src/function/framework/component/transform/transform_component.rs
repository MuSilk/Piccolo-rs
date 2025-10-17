use std::cell::RefCell;

use nalgebra_glm::Mat4;
use reflection::reflection_derive::ReflectWhiteListFields;

use crate::{core::math::transform::Transform, function::framework::component::component::{Component, ComponentTrait}};


#[derive(Clone, Default, ReflectWhiteListFields)]
pub struct TransformComponent {
    m_component: Component,
    #[meta]
    m_transform: Transform,
    m_transform_buffer: [Transform; 2],
    m_current_index: usize,
    m_next_index: usize,
}

impl ComponentTrait for TransformComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
}

impl TransformComponent {
    pub fn get_matrix(&self) -> Mat4 {
        self.m_transform_buffer[self.m_current_index].get_matrix()
    }
}