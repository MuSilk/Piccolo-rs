use serde::{Deserialize, Serialize};

use crate::{function::{framework::{component::component::{Component, ComponentTrait}}}, resource::res_type::{components::mesh::MeshComponentRes}};


pub enum FaceDirection {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub const FACE_DIRECTION_OFFSETS: [(i32, i32, i32); 6] = [
    ( 0, 0, 1),  // Top
    ( 0, 0,-1),  // Bottom
    (-1, 0, 0),  // Left
    ( 1, 0, 0),  // Right
    ( 0, 1, 0),  // Front
    ( 0,-1, 0),  // Back
];

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum BlockType {
    #[default]
    Air,
    Dirt,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Block {
    #[serde(skip)]
    pub m_component: Component,
    pub block_type: BlockType,
    pub m_mesh_res: MeshComponentRes,
}

#[typetag::serde]
impl ComponentTrait for Block {
    fn as_any(&self) ->  &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_component(&self) -> &Component {
        &self.m_component
    }

    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }

    fn clone_box(&self) -> Box<dyn ComponentTrait> {
        Box::new(self.clone())
    }
}