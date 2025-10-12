use std::{collections::HashMap, ptr::copy_nonoverlapping};
use anyhow::Result;
use nalgebra_glm::Vec3;
use vulkanalia::{prelude::v1_0::*};

use crate::function::render::{interface::vulkan::vulkan_rhi::VulkanRHI, render_camera::RenderCamera, render_common::{MeshPreframeStorageBufferObject, VulkanMesh}, render_entity::{self, RenderEntity}, render_mesh::{MeshVertex, VulkanMeshVertexPosition}, render_resource_base::RenderResourceBase, render_scene::RenderScene, render_type::{MeshVertexDataDefinition, RenderMeshData}};


#[derive(Default)]
pub struct RenderResource{
    pub m_base:RenderResourceBase,
    pub m_mesh_perframe_storage_buffer_object: MeshPreframeStorageBufferObject,

    pub m_vulkan_meshes: HashMap<usize, VulkanMesh>,
}

impl RenderResource {
    pub fn update_per_frame_buffer(&mut self, render_scene: &RenderScene, camera: &RenderCamera){
        let view_matrix = camera.get_view_matrix();
        let proj_matrix = camera.get_pers_proj_matrix();
        let camera_position = camera.position();
        let proj_view_matrix = proj_matrix * view_matrix;

        self.m_mesh_perframe_storage_buffer_object.proj_view_matrix = proj_view_matrix;
        self.m_mesh_perframe_storage_buffer_object.camera_position = *camera_position;
        
    }

    pub fn upload_game_object_render_resource(&mut self, rhi: &VulkanRHI, render_entity: &RenderEntity, mesh_data: &RenderMeshData){
        self.get_or_create_vulkan_mesh(rhi, render_entity, mesh_data);
    }
}

impl RenderResource {
    fn get_or_create_vulkan_mesh(&mut self, rhi: &VulkanRHI, entity: &RenderEntity, mesh_data: &RenderMeshData) -> &VulkanMesh {
        let assetid = entity.m_mesh_asset_id;

        if let None = self.m_vulkan_meshes.get(&assetid) {
            self.m_vulkan_meshes.insert(assetid, VulkanMesh::default());
            let now_mesh = self.m_vulkan_meshes.get_mut(&assetid).unwrap();

            let index_buffer_size = mesh_data.m_static_mesh_data.m_index_buffer.m_data.len();
            let index_buffer_data = &mesh_data.m_static_mesh_data.m_index_buffer.m_data;

            let vertex_buffer_size = mesh_data.m_static_mesh_data.m_vertex_buffer.m_data.len();
            let vertex_buffer_data = &mesh_data.m_static_mesh_data.m_vertex_buffer.m_data;

            if mesh_data.m_skeleton_binding_buffer.m_data.len() > 0{
                unimplemented!();
            }
            else{
                let vertex_buffer_data: &[MeshVertexDataDefinition] = unsafe{
                    std::slice::from_raw_parts(
                        vertex_buffer_data.as_ptr().cast(),
                        vertex_buffer_size / std::mem::size_of::<MeshVertexDataDefinition>(),
                    )
                };
                Self::update_mesh_data(
                    rhi,
                    false,
                    index_buffer_size as u32, 
                    index_buffer_data, 
                    vertex_buffer_data, 
                    now_mesh
                );
            }
        }

        let now_mesh = self.m_vulkan_meshes.get(&assetid).unwrap();
        &now_mesh
    }

    fn update_mesh_data(
        rhi: &VulkanRHI,
        enable_vertex_blending: bool,
        index_buffer_size: u32,
        index_buffer_data: &[u8],
        vertex_buffer_data: &[MeshVertexDataDefinition],
        now_mesh: &mut VulkanMesh,
    ) -> Result<()> {
        now_mesh.enable_vertex_blending = enable_vertex_blending;
        now_mesh.mesh_vertex_count = vertex_buffer_data.len() as u32;
        Self::update_vertex_buffer(rhi, enable_vertex_blending, vertex_buffer_data, now_mesh)?;
        now_mesh.mesh_index_count = index_buffer_size / std::mem::size_of::<u16>() as u32;
        Self::update_index_buffer(rhi, index_buffer_size, index_buffer_data, now_mesh)?;
        Ok(())
    }

    fn update_vertex_buffer(
        rhi: &VulkanRHI,
        enable_vertex_blending: bool,
        vertex_buffer_data: &[MeshVertexDataDefinition],
        now_mesh: &mut VulkanMesh,
    ) -> Result<()> {
        if enable_vertex_blending{
            unimplemented!();
        }
        else{
            let vertex_position_buffer_size = size_of::<VulkanMeshVertexPosition>() * vertex_buffer_data.len();
            let (staging_buffer, staging_memory) = rhi.create_buffer(
                vertex_position_buffer_size as u64, 
                vk::BufferUsageFlags::TRANSFER_SRC, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let staging_buffer_data = rhi.map_memory(
                staging_memory, 0, vertex_position_buffer_size as u64, vk::MemoryMapFlags::empty()
            )?;
            let mesh_vertex_positions = unsafe{
                std::slice::from_raw_parts_mut::<VulkanMeshVertexPosition>(
                    staging_buffer_data as *mut VulkanMeshVertexPosition,
                    vertex_buffer_data.len(),
                )
            };
            for vertex_index in 0..vertex_buffer_data.len() {
                let normal = Vec3::new(
                    vertex_buffer_data[vertex_index].nx,
                    vertex_buffer_data[vertex_index].ny,
                    vertex_buffer_data[vertex_index].nz,
                );
                let tangent = Vec3::new(
                    vertex_buffer_data[vertex_index].tx,
                    vertex_buffer_data[vertex_index].ty,
                    vertex_buffer_data[vertex_index].tz,
                );
                mesh_vertex_positions[vertex_index].position = Vec3::new(
                    vertex_buffer_data[vertex_index].x,
                    vertex_buffer_data[vertex_index].y,
                    vertex_buffer_data[vertex_index].z,
                );
            }
            rhi.unmap_memory(staging_memory);
            (now_mesh.mesh_vertex_position_buffer, now_mesh.mesh_vertex_position_buffer_allocation) = rhi.create_buffer(
                vertex_position_buffer_size as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            rhi.copy_buffer(staging_buffer, now_mesh.mesh_vertex_position_buffer,0,0, vertex_position_buffer_size as u64)?;
            rhi.destroy_buffer(staging_buffer);
            rhi.free_memory(staging_memory);
        }
        Ok(())
    } 
    fn update_index_buffer(rhi: &VulkanRHI, index_buffer_size: u32, index_buffer_data: &[u8], now_mesh: &mut VulkanMesh) -> Result<()>{
        let buffer_size = index_buffer_size as u64;
        let (staging_buffer, staging_memory) = rhi.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let staging_buffer_data = rhi.map_memory(staging_memory, 0, buffer_size, vk::MemoryMapFlags::empty())?;
        unsafe{
            copy_nonoverlapping(index_buffer_data.as_ptr().cast(), staging_buffer_data, buffer_size as usize);
        }
        rhi.unmap_memory(staging_memory);
        
        let (buffer, memory) = rhi.create_buffer(
            buffer_size, 
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        rhi.copy_buffer(staging_buffer, buffer, 0, 0, buffer_size)?;
        rhi.destroy_buffer(staging_buffer);
        rhi.free_memory(staging_memory);

        now_mesh.mesh_index_buffer = buffer;
        now_mesh.mesh_index_buffer_allocation = memory;

        Ok(())
    }
}