use serde::{Deserialize, Serialize};

use crate::{core::math::{matrix4::Matrix4x4, transform::Transform}, function::framework::component::component::{Component, ComponentTrait}};


#[derive(Clone, Default, Serialize, Deserialize)]
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

#[typetag::serde]
impl ComponentTrait for TransformComponent {
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

impl TransformComponent {
    pub fn get_matrix(&self) -> Matrix4x4 {
        self.m_transform_buffer[self.m_current_index].get_matrix()
    }
}