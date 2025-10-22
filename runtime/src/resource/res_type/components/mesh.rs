use crate::core::math::transform::Transform;

#[derive(Clone, Default)]
pub struct SubMeshRes {
    pub m_obj_file_ref: String,
    pub m_transform: Transform,
    pub m_material: String,
}

#[derive(Clone, Default)]
pub struct MeshComponentRes {
    pub m_sub_meshs: Vec<SubMeshRes>,
}