use crate::core::math::{axis_aligned::AxisAlignedBox, matrix4::Matrix4x4, vector3::Vector3, vector4::Vector4};

pub struct RenderEntity {
    pub m_instance_id: u32,
    pub m_model_matrix: Matrix4x4,
    
    pub m_mesh_asset_id: usize,
    pub m_enable_vertex_blending: bool,
    pub m_joint_matrices: Vec<Matrix4x4>,
    pub m_bounding_box: AxisAlignedBox,

    pub m_material_asset_id: usize,
    pub m_blend: bool,
    pub m_double_sided: bool,
    pub m_base_color_factor: Vector4,
    pub m_metallic_factor: f32,
    pub m_roughness_factor: f32,
    pub m_normal_scale: f32,
    pub m_occlusion_strength: f32,
    pub m_emissive_factor: Vector3,
}

impl Default for RenderEntity {
    fn default() -> Self {
        Self {
            m_instance_id: 0,
            m_model_matrix: Matrix4x4::identity(),
            
            m_mesh_asset_id: 0,
            m_enable_vertex_blending: false,
            m_joint_matrices: Default::default(),
            m_bounding_box: Default::default(),

            m_material_asset_id: 0,
            m_blend: false,
            m_double_sided: false,
            m_base_color_factor: Vector4::new(1.0, 1.0, 1.0, 1.0),
            m_metallic_factor: 1.0,
            m_roughness_factor: 1.0,
            m_normal_scale: 1.0,
            m_occlusion_strength: 1.0,
            m_emissive_factor: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}