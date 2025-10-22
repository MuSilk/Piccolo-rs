use serde::{Deserialize, Serialize};

use crate::core::math::transform::Transform;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SubMeshRes {
    pub m_obj_file_ref: String,
    pub m_transform: Transform,
    pub m_material: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MeshComponentRes {
    pub m_sub_meshs: Vec<SubMeshRes>,
}