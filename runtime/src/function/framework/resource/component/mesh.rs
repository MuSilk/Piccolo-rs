use serde::{Deserialize, Serialize};

use crate::{core::math::transform::Transform, function::framework::resource::resource::Resource};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SubMeshRes {
    pub m_obj_file_ref: Option<String>,
    pub m_transform: Transform,
    pub m_material: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MeshComponentRes {
    pub m_sub_meshs: Vec<SubMeshRes>,
}

#[typetag::serde]
impl Resource for MeshComponentRes {

}