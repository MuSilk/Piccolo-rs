#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MaterialRes {
    #[serde(rename = "base_colour_texture_file")]
    pub m_base_colour_texture_file : String,
    #[serde(rename = "metallic_roughness_texture_file")]
    pub m_metallic_roughness_texture_file : String,
    #[serde(rename = "normal_texture_file")]
    pub m_normal_texture_file : String,
    #[serde(rename = "occlusion_texture_file")]
    pub m_occlusion_texture_file : String,
    #[serde(rename = "emissive_texture_file")]
    pub m_emissive_texture_file : String,
}