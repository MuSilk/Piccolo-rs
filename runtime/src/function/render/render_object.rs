use nalgebra_glm::Mat4;
use reflection::reflection_derive::{ReflectFields, ReflectWhiteListFields};

use crate::function::framework::object::object_id_allocator::GObjectID;


#[derive(Clone, ReflectFields)]
pub struct GameObjectMeshDesc {
    pub m_mesh_file: String,
}

#[derive(Clone, ReflectFields)]
pub struct SkeletonBindingDesc {
    pub m_skeleton_binding_file: String,
}

#[derive(Clone, ReflectWhiteListFields)]
pub struct SkeletonAnimationResultTransform {
    pub m_matrix: Mat4,
}

#[derive(Clone, ReflectFields)]
pub struct SkeletonAnimationResult {
    pub m_transforms: Vec<SkeletonAnimationResultTransform>,
}

#[derive(Clone, ReflectFields)]
pub struct GameObjectMaterialDesc{
    pub m_base_color_texture_file: String,
    pub m_metallic_roughness_texture_file: String,
    pub m_normal_texture_file: String,
    pub m_occlusion_texture_file: String,
    pub m_emissive_texture_file: String,
    pub m_with_texture: bool
}

#[derive(ReflectWhiteListFields)]
pub struct GameObjectTransformDesc {
    pub m_transform_matrix: Mat4,
}

#[derive(ReflectFields)]
pub struct GameObjectPartDesc {
    pub m_mesh_desc: GameObjectMeshDesc,
    pub m_material_desc: GameObjectMaterialDesc,
    pub m_with_animation: bool,
    pub m_skeleton_binding_desc: SkeletonBindingDesc,
    pub m_skeleton_animation_result: SkeletonAnimationResult,
}

impl GameObjectPartDesc {
    pub const K_INVALID_PART_ID: usize = usize::MAX; 
}

pub struct GameObjectPartId {
    m_go_id: GObjectID,
    m_part_id: usize,
}

pub struct GameObjectDesc {
    m_go_id : GObjectID,
    m_object_parts: Vec<GameObjectPartDesc>,
} 
