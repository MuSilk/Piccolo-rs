use std::{array, rc::{Weak}};

use nalgebra_glm::{Mat4, Vec3, Vec4};
use vulkanalia::{prelude::v1_0::*};


const S_MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT: usize = 64;
const S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT: usize = 1024;
const S_MAX_POINT_LIGHT_COUNT: usize                = 15;

#[derive(Clone ,Default)]
pub struct VulkanSceneDirectionalLight {
    pub direction : Vec3,
    pub _padding_direction: f32,
    pub color : Vec3,
    pub _padding_color: f32,
}

pub struct VulkanScenePointLight {
    pub position : Vec3,
    pub radius: f32,
    pub intensity: Vec3,
    pub _padding_intensity: f32,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct MeshPerframeStorageBufferObject {
    pub proj_view_matrix : Mat4,
    pub camera_position : Vec3,
    pub _padding_camera_position: f32,
    pub ambient_light : Vec3,
    pub _padding_ambient_light: f32,
    pub point_light_num: u32,
    pub _padding_point_light_num_1: u32,
    pub _padding_point_light_num_2: u32,
    pub _padding_point_light_num_3: u32,
    pub scene_point_lights: [Vec4; S_MAX_POINT_LIGHT_COUNT],
    pub scene_directional_light: VulkanSceneDirectionalLight,
    pub directional_light_proj_view: Mat4
}
#[repr(C)]
#[derive(Clone, Default)]
pub struct VulkanMeshInstance {
    pub enable_vertex_blending: f32,
    pub _padding_enable_vertex_blending_1: f32,
    pub _padding_enable_vertex_blending_2: f32,
    pub _padding_enable_vertex_blending_3: f32,
    pub model_matrix: Mat4,
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshPerdrawcallStorageBufferObject {
    pub mesh_instances: [VulkanMeshInstance; S_MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT]
}

impl Default for MeshPerdrawcallStorageBufferObject {
    fn default() -> Self {
        Self {
            mesh_instances: array::from_fn(|_| VulkanMeshInstance::default()),
        }
    }
}

#[repr(C)]
pub struct MeshPerMaterialUniformBufferObject {
    pub base_color_factor: Vec4,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,

    pub emissive_factor: Vec3,
    pub is_blend: u32,
    pub id_double_sided: u32,
}

#[derive(Clone, Default)]
pub struct VulkanMesh {
    pub enable_vertex_blending: bool,

    pub mesh_vertex_count : u32,

    pub mesh_vertex_position_buffer: vk::Buffer,
    pub mesh_vertex_position_buffer_allocation: vk::DeviceMemory,

    pub mesh_vertex_varying_enable_blending_buffer: vk::Buffer,
    pub mesh_vertex_varying_enable_blending_buffer_allocation: vk::DeviceMemory,

    pub mesh_vertex_joint_binding_buffer: vk::Buffer,
    pub mesh_vertex_joint_binding_buffer_allocation: vk::DeviceMemory,

    pub mesh_vertex_blending_descriptor_set: vk::DescriptorSet,

    pub mesh_vertex_varying_buffer: vk::Buffer,
    pub mesh_vertex_varying_buffer_allocation: vk::DeviceMemory,

    pub mesh_index_count: u32,

    pub mesh_index_buffer: vk::Buffer,
    pub mesh_index_buffer_allocation: vk::DeviceMemory,
}

#[derive(Default)]
pub struct VulkanPBRMaterial {
    pub base_color_texture_image: vk::Image,
    pub base_color_image_view: vk::ImageView,
    pub base_color_image_allocation: vk::DeviceMemory,

    pub metallic_roughness_texture_image: vk::Image,
    pub metallic_roughness_image_view: vk::ImageView,
    pub metallic_roughness_image_allocation: vk::DeviceMemory,

    pub normal_texture_image: vk::Image,
    pub normal_image_view: vk::ImageView,
    pub normal_image_allocation: vk::DeviceMemory,

    pub occlusion_texture_image: vk::Image,
    pub occlusion_image_view: vk::ImageView,
    pub occlusion_image_allocation: vk::DeviceMemory,

    pub emissive_texture_image: vk::Image,
    pub emissive_image_view: vk::ImageView,
    pub emissive_image_allocation: vk::DeviceMemory,

    pub material_uniform_buffer: vk::Buffer,
    pub material_uniform_buffer_allocation: vk::DeviceMemory,

    pub material_descriptor_set: vk::DescriptorSet,
}

#[derive(Clone, Default)]
pub struct RenderMeshNode {
    pub model_matrix: Mat4,
    pub joint_matrices: Vec<Mat4>,
    pub ref_mesh: Weak<VulkanMesh>,
    pub ref_material: Weak<VulkanPBRMaterial>,
    pub node_id: u32,
    pub enable_vertex_blending: bool,
}

pub struct TextureDataToUpdate<'a>{
    pub base_color_image_pixels: &'a [u8],
    pub base_color_image_width: u32,
    pub base_color_image_height: u32,
    pub base_color_image_format: vk::Format,

    pub metallic_roughness_image_pixels: &'a [u8],
    pub metallic_roughness_image_width: u32,
    pub metallic_roughness_image_height: u32,
    pub metallic_roughness_image_format: vk::Format,

    pub normal_roughness_image_pixels: &'a [u8],
    pub normal_roughness_image_width: u32,
    pub normal_roughness_image_height: u32,
    pub normal_roughness_image_format: vk::Format,

    pub occlusion_image_pixels: &'a [u8],
    pub occlusion_image_width: u32,
    pub occlusion_image_height: u32,
    pub occlusion_image_format: vk::Format,

    pub emissive_image_pixels: &'a [u8],
    pub emissive_image_width: u32,
    pub emissive_image_height: u32,
    pub emissive_image_format: vk::Format,

    pub now_material: &'a mut VulkanPBRMaterial,
}