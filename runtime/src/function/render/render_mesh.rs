use std::mem::offset_of;

use vulkanalia::{prelude::v1_0::*};

use crate::core::math::{vector2::Vector2, vector3::Vector3, vector4::Vector4};

#[repr(C)]
pub struct VulkanMeshVertexPosition{
    pub position: Vector3,
}

#[repr(C)]
pub struct VulkanMeshVertexVaryingEnableBlending{
    pub normal: Vector3,
    pub tangent: Vector3,
}

#[repr(C)]
pub struct VulkanMeshVertexVarying{
    pub texcoord: Vector2,
}

#[repr(C)]
pub struct VulkanMeshVertexJointBinding{
    pub indices: [u32; 4],
    pub weights: Vector4,
}

pub struct MeshVertex{

}

impl MeshVertex{
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 3]{
        [
            vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(std::mem::size_of::<VulkanMeshVertexPosition>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
            vk::VertexInputBindingDescription::builder()
                .binding(1)
                .stride(std::mem::size_of::<VulkanMeshVertexVaryingEnableBlending>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
            vk::VertexInputBindingDescription::builder()
                .binding(2)
                .stride(std::mem::size_of::<VulkanMeshVertexVarying>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
        ]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4]{
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(VulkanMeshVertexPosition, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(1)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(VulkanMeshVertexVaryingEnableBlending, normal) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(1)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(VulkanMeshVertexVaryingEnableBlending, tangent) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(2)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(VulkanMeshVertexVarying, texcoord) as u32)
                .build(),
        ]
    }
}