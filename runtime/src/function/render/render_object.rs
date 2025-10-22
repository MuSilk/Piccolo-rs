use crate::{core::math::matrix4::Matrix4x4, function::framework::object::object_id_allocator::{GObjectID, K_INVALID_GOBJECT_ID}};


#[derive(Clone, Default)]
pub struct GameObjectMeshDesc {
    pub m_mesh_file: String,
}

#[derive(Clone, Default)]
pub struct SkeletonBindingDesc {
    pub m_skeleton_binding_file: String,
}

#[derive(Clone, Default)]
pub struct SkeletonAnimationResultTransform {
    pub m_matrix: Matrix4x4,
}

#[derive(Clone, Default)]
pub struct SkeletonAnimationResult {
    pub m_transforms: Vec<SkeletonAnimationResultTransform>,
}

#[derive(Clone, Default)]
pub struct GameObjectMaterialDesc{
    pub m_base_color_texture_file: String,
    pub m_metallic_roughness_texture_file: String,
    pub m_normal_texture_file: String,
    pub m_occlusion_texture_file: String,
    pub m_emissive_texture_file: String,
    pub m_with_texture: bool
}

#[derive(Clone, Default)]
pub struct GameObjectTransformDesc {
    pub m_transform_matrix: Matrix4x4,
}

#[derive(Clone, Default)]
pub struct GameObjectPartDesc {
    pub m_mesh_desc: GameObjectMeshDesc,
    pub m_material_desc: GameObjectMaterialDesc,
    pub m_transform_desc: GameObjectTransformDesc,
    pub m_with_animation: bool,
    pub m_skeleton_binding_desc: SkeletonBindingDesc,
    pub m_skeleton_animation_result: SkeletonAnimationResult,
}

impl GameObjectPartDesc {
    pub const K_INVALID_PART_ID: usize = usize::MAX; 
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameObjectPartId {
    pub m_go_id: GObjectID,
    pub m_part_id: usize,
}

impl Default for GameObjectPartId {
    fn default() -> Self {
        Self {
            m_go_id : K_INVALID_GOBJECT_ID,
            m_part_id: GameObjectPartDesc::K_INVALID_PART_ID,
        }
    }
}

#[derive(Clone)]
pub struct GameObjectDesc {
    m_go_id : GObjectID,
    m_object_parts: Vec<GameObjectPartDesc>,
} 

impl GameObjectDesc {
    pub fn new(go_id: GObjectID, object_parts: Vec<GameObjectPartDesc>) -> Self {
        Self {
            m_go_id : go_id,
            m_object_parts: object_parts,
        }
    }

    pub fn get_id(&self) -> GObjectID {
        self.m_go_id
    }

    pub fn get_object_parts(&self) -> &[GameObjectPartDesc] {
        self.m_object_parts.as_slice()
    }
}
