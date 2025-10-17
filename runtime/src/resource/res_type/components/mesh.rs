use reflection::reflection_derive::ReflectFields;

use crate::core::math::transform::Transform;

#[derive(ReflectFields, Clone, Default)]
pub struct SubMeshRes {
    pub m_obj_file_ref: String,
    pub m_transform: Transform,
    pub m_material: String,
}

#[derive(ReflectFields, Clone, Default)]
pub struct MeshComponentRes {
    pub m_sub_meshs: Vec<SubMeshRes>,
}