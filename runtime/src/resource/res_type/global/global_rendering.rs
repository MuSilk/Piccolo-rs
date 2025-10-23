use crate::{core::math::vector3::Vector3, resource::res_type::data::camera_config::CameraConfig};


#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SkyBoxIrradianceMap {
    pub negative_x_map: String,
    pub positive_x_map: String,
    pub negative_y_map: String,
    pub positive_y_map: String,
    pub negative_z_map: String,
    pub positive_z_map: String,
}


#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SkyBoxSpecularMap {
    pub negative_x_map: String,
    pub positive_x_map: String,
    pub negative_y_map: String,
    pub positive_y_map: String,
    pub negative_z_map: String,
    pub positive_z_map: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DirectionalLight {
    pub direction: Vector3,
    pub color: Vector3
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GlobalRenderingRes {
    pub enable_fxaa: bool,
    pub skybox_irradiance_map: SkyBoxIrradianceMap,
    pub skybox_specular_map: SkyBoxSpecularMap,
    pub brdf_map: String,
    pub color_grading_map: String,

    pub sky_color: Vector3,
    pub ambient_light: Vector3,
    pub camera_config: CameraConfig,
    pub directional_light: DirectionalLight,
}