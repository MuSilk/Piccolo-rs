use reflection::reflection_derive::ReflectFields;

use crate::core::math::transform::Transform;

#[derive(ReflectFields, Clone)]
struct SubMeshRes {
    m_obj_file_ref: String,
    m_transform: Transform,
    m_material: String,
}

#[derive(ReflectFields, Clone)]
pub struct MeshComponentRes {
    m_sub_meshs: Vec<SubMeshRes>,
}