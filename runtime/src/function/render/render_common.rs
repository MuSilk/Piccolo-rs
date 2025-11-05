use std::{array, rc::{Rc, Weak}};

use vulkanalia::{prelude::v1_0::*};

use crate::{core::math::{matrix4::Matrix4x4, vector3::Vector3, vector4::Vector4}};

pub const S_POINT_LIGHT_SHADOW_MAP_DIMENSION: u32 = 2048;
pub const S_DIRECTIONAL_LIGHT_SHADOW_MAP_DIMENSION: u32 = 4096;


pub const MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT: usize = 64;
const S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT: usize = 1024;
pub const S_MAX_POINT_LIGHT_COUNT: usize                = 15;

#[derive(Clone ,Default)]
pub struct VulkanSceneDirectionalLight {
    pub direction : Vector3,
    pub _padding_direction: f32,
    pub color : Vector3,
    pub _padding_color: f32,
}

#[derive(Clone ,Default)]
pub struct VulkanScenePointLight {
    pub position : Vector3,
    pub radius: f32,
    pub intensity: Vector3,
    pub _padding_intensity: f32,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct MeshPerframeStorageBufferObject {
    pub proj_view_matrix : Matrix4x4,
    pub camera_position : Vector3,
    pub _padding_camera_position: f32,
    pub ambient_light : Vector3,
    pub _padding_ambient_light: f32,
    pub point_light_num: u32,
    pub _padding_point_light_num_1: u32,
    pub _padding_point_light_num_2: u32,
    pub _padding_point_light_num_3: u32,
    pub scene_point_lights: [VulkanScenePointLight; S_MAX_POINT_LIGHT_COUNT],
    pub scene_directional_light: VulkanSceneDirectionalLight,
    pub directional_light_proj_view: Matrix4x4
}
#[repr(C)]
#[derive(Clone, Default)]
pub struct VulkanMeshInstance {
    pub enable_vertex_blending: f32,
    _padding_enable_vertex_blending_1: f32,
    _padding_enable_vertex_blending_2: f32,
    _padding_enable_vertex_blending_3: f32,
    pub model_matrix: Matrix4x4,
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshPerdrawcallStorageBufferObject {
    pub mesh_instances: [VulkanMeshInstance; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT]
}

impl Default for MeshPerdrawcallStorageBufferObject {
    fn default() -> Self {
        Self {
            mesh_instances: array::from_fn(|_| VulkanMeshInstance::default()),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshPerdrawcallVertexBlendingStorageBufferObject {
    pub joint_matrices: [VulkanMeshInstance; S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT * MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT]
}

impl Default for MeshPerdrawcallVertexBlendingStorageBufferObject {
    fn default() -> Self {
        Self {
            joint_matrices: array::from_fn(|_| VulkanMeshInstance::default()),
        }
    }
}

#[repr(C)]
pub struct MeshPerMaterialUniformBufferObject {
    pub base_color_factor: Vector4,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,

    pub emissive_factor: Vector3,
    pub is_blend: u32,
    pub id_double_sided: u32,
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct MeshPointLightShadowPerframeStorageBufferObject {
    pub point_light_num: u32,
    _padding_point_light_num_1: u32,
    _padding_point_light_num_2: u32,
    _padding_point_light_num_3: u32,
    pub point_lights_position_and_radius: [Vector4; S_MAX_POINT_LIGHT_COUNT],
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshPointLightShadowPerdrawcallStorageBufferObject {
    pub mesh_instances: [VulkanMeshInstance; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT]
}

impl Default for MeshPointLightShadowPerdrawcallStorageBufferObject {
    fn default() -> Self {
        Self {
            mesh_instances: array::from_fn(|_| VulkanMeshInstance::default()),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshPointLightShadowPerdrawcallVertexBlendingStorageBufferObject {
    pub joint_matrices: [Matrix4x4; S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT * S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT],
}

impl Default for MeshPointLightShadowPerdrawcallVertexBlendingStorageBufferObject {
    fn default() -> Self {
        Self {
            joint_matrices: array::from_fn(|_| Matrix4x4::default()),
        }
    }
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct MeshDirectionalLightShadowPerframeStorageBufferObject {
    pub light_proj_view: Matrix4x4,
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshDirectionalLightShadowPerdrawcallStorageBufferObject {
    pub mesh_instances: [VulkanMeshInstance; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT]
}

impl Default for MeshDirectionalLightShadowPerdrawcallStorageBufferObject {
    fn default() -> Self {
        Self {
            mesh_instances: array::from_fn(|_| VulkanMeshInstance::default()),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct MeshDirectionalLightShadowPerdrawcallVertexBlendingStorageBufferObject {
    pub joint_matrices: [Matrix4x4; S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT * S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT],
}

impl Default for MeshDirectionalLightShadowPerdrawcallVertexBlendingStorageBufferObject {
    fn default() -> Self {
        Self {
            joint_matrices: array::from_fn(|_| Matrix4x4::default()),
        }
    }
}

#[derive(Clone, Default)]
pub struct MeshInefficientPickPerframeStorageBufferObject {
    pub proj_view_matrix: Matrix4x4,
    pub rt_width: u32,
    pub rt_height: u32,
}

pub struct MeshInefficientPickPerdrawcallStorageBufferObject {
    pub model_matrix: [Matrix4x4; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT],
    pub node_ids: [u32; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT],
    pub enable_vertex_blending: [i32; MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT],
}

impl Default for MeshInefficientPickPerdrawcallStorageBufferObject {
    fn default() -> Self {
        Self {
            model_matrix: array::from_fn(|_| Matrix4x4::default()),
            node_ids: array::from_fn(|_| 0),
            enable_vertex_blending: array::from_fn(|_| 0),
        }
    }
}

pub struct MeshInefficientPickPerdrawcallVertexBlendingStorageBufferObject {
    pub joint_matrices: [Matrix4x4; S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT * S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT],
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
    pub mesh_index_type: vk::IndexType,

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
    pub model_matrix: Rc<Matrix4x4>,
    pub joint_matrices: Vec<Matrix4x4>,
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