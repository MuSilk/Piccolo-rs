use nalgebra_glm::{Mat4, Vec3, Vec4};

use crate::core::math::axis_aligned::AxisAlignedBox;

pub struct RenderEntity {
    pub m_instance_id: u32,
    pub m_model_matrix: Mat4,
    
    pub m_mesh_asset_id: usize,
    pub m_enable_vertex_blending: bool,
    pub m_joint_matrices: Vec<Mat4>,
    pub m_bounding_box: AxisAlignedBox,

    pub m_material_asset_id: usize,
    pub m_blend: bool,
    pub m_double_sided: bool,
    pub m_base_factor: Vec4,
    pub m_metallic_factor: f32,
    pub m_roughness_factor: f32,
    pub m_normal_scale: f32,
    pub m_occlusion_strength: f32,
    pub m_emissive_factor: Vec3,
}