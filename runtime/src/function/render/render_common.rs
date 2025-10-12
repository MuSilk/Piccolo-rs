use nalgebra_glm::{Mat4, Vec3, Vec4};
use vulkanalia::{prelude::v1_0::*};


const S_MESH_PER_DRAWCALL_MAX_INSTANCE_COUNT: usize = 64;
const S_MESH_VERTEX_BLENDING_MAX_JOINT_COUNT: usize = 1024;
const S_MAX_POINT_LIGHT_COUNT: usize                = 15;

#[derive(Default)]
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

#[derive(Default)]
pub struct MeshPreframeStorageBufferObject {
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

#[derive(Default)]
pub struct VulkanMesh {
    pub enable_vertex_blending: bool,

    pub mesh_vertex_count : u32,

    pub mesh_vertex_position_buffer: vk::Buffer,
    pub mesh_vertex_position_buffer_allocation: vk::DeviceMemory,

    mesh_vertex_varying_enable_blending_buffer: vk::Buffer,
    mesh_vertex_varying_enable_blending_buffer_allocation: vk::DeviceMemory,

    mesh_vertex_joint_binding_buffer: vk::Buffer,
    mesh_vertex_joint_binding_buffer_allocation: vk::DeviceMemory,

    mesh_vertex_blending_descriptor_set: vk::DescriptorSet,

    mesh_vertex_varying_buffer: vk::Buffer,
    mesh_vertex_varying_buffer_allocation: vk::DeviceMemory,

    pub mesh_index_count: u32,

    pub mesh_index_buffer: vk::Buffer,
    pub mesh_index_buffer_allocation: vk::DeviceMemory,
}
