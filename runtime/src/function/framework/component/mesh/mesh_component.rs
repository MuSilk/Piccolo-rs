use reflection::reflection_derive::ReflectWhiteListFields;

use crate::{function::render::render_object::GameObjectPartDesc, resource::res_type::components::mesh::MeshComponentRes};



#[derive(ReflectWhiteListFields)]
pub struct MeshComponent {
    #[meta]
    m_raw_meshs: MeshComponentRes,
    m_raw_meshes: Vec<GameObjectPartDesc>,
}