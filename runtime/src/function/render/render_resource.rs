use std::{cell::RefCell, collections::HashMap, os::raw::c_void, ptr::copy_nonoverlapping, rc::Rc};
use anyhow::Result;
use itertools::Itertools;
use vulkanalia::{prelude::v1_0::*};

use crate::{core::math::{vector2::Vector2, vector3::Vector3}, function::render::{interface::vulkan::vulkan_rhi::{self, VulkanRHI}, render_camera::RenderCamera, render_common::{MeshPerMaterialUniformBufferObject, MeshPerframeStorageBufferObject, TextureDataToUpdate, VulkanMesh, VulkanPBRMaterial}, render_entity::RenderEntity, render_mesh::{VulkanMeshVertexPosition, VulkanMeshVertexVarying, VulkanMeshVertexVaryingEnableBlending}, render_resource_base::RenderResourceBase, render_scene::RenderScene, render_swap_context::LevelResourceDesc, render_type::{MeshVertexDataDefinition, RHISamplerType, RenderMaterialData, RenderMeshData, TextureData}}};

#[derive(Default)]
struct IBLResource {
    _brdf_lut_texture_image: vk::Image,
    _brdf_lut_texture_image_view: vk::ImageView,
    _brdf_lut_texture_sampler: vk::Sampler,
    _brdf_lut_texture_image_allocation: vk::DeviceMemory,

    _irradiance_texture_image: vk::Image,
    _irradiance_texture_image_view: vk::ImageView,
    _irradiance_texture_sampler: vk::Sampler,
    _irradiance_texture_image_allocation: vk::DeviceMemory,

    _specular_texture_image: vk::Image,
    _specular_texture_image_view: vk::ImageView,
    _specular_texture_sampler: vk::Sampler,
    _specular_texture_image_allocation: vk::DeviceMemory,
}

struct IBLResourceData {

}

#[derive(Default)]
pub struct ColorGradingResource {
    pub _color_grading_lut_texture_image: vk::Image,
    pub _color_grading_lut_texture_image_view: vk::ImageView,
    pub _color_grading_lut_texture_image_allocation: vk::DeviceMemory,
}

struct ColorGradingResourceData {

}

#[derive(Default)]
pub struct StorageBuffer {
    pub _min_uniform_buffer_offset_alignment: u32,
    pub _min_storage_buffer_offset_alignment: u32,
    pub _max_storage_buffer_range: u32,
    pub _non_coherent_atom_size: u32,

    pub _global_upload_ringbuffer: vk::Buffer,
    pub _global_upload_ringbuffer_memory: vk::DeviceMemory,
    pub _global_upload_ringbuffer_pointer: *mut c_void,
    pub _global_upload_ringbuffers_begin: Vec<u32>,
    pub _global_upload_ringbuffers_end: Vec<u32>,
    pub _global_upload_ringbuffers_size: Vec<u32>,
}

#[derive(Default)]
pub struct GlobalRenderResource {
    _ibl_resource: IBLResource,
    pub _color_grading_resource: ColorGradingResource,
    pub _storage_buffer: StorageBuffer,
}

#[derive(Clone, Default)]
pub struct RenderResource{
    pub m_base:RenderResourceBase,

    pub m_global_render_resource: Rc<RefCell<GlobalRenderResource>>,

    pub m_mesh_perframe_storage_buffer_object: MeshPerframeStorageBufferObject,

    pub m_vulkan_meshes: HashMap<usize, Rc<VulkanMesh>>,
    pub m_vulkan_pbr_materials: HashMap<usize, Rc<VulkanPBRMaterial>>,

    pub m_material_descriptor_set_layout: vk::DescriptorSetLayout,
}

impl RenderResource {
    pub fn reset_ring_buffer_offset(&mut self, current_frame_index: usize) {
        let mut resource = self.m_global_render_resource.borrow_mut();
        resource._storage_buffer._global_upload_ringbuffers_end[current_frame_index] =
            resource._storage_buffer._global_upload_ringbuffers_begin[current_frame_index];
    }

    pub fn upload_global_render_resource(&mut self, rhi: &VulkanRHI, level_resource_desc: &LevelResourceDesc) {
        self.create_and_map_storage_buffer(rhi);
        
        let skybox_irradiance_map = &level_resource_desc.m_ibl_resource_desc.m_skybox_irradiance_map;
        let irradiace_pos_x_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.positive_x_map, 4).unwrap();
        let irradiace_neg_x_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.negative_x_map, 4).unwrap();
        let irradiace_pos_y_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.positive_y_map, 4).unwrap();
        let irradiace_neg_y_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.negative_y_map, 4).unwrap();
        let irradiace_pos_z_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.positive_z_map, 4).unwrap();
        let irradiace_neg_z_map = RenderResourceBase::load_texture_hdr(&skybox_irradiance_map.negative_z_map, 4).unwrap();

        let skybox_specular_map = &level_resource_desc.m_ibl_resource_desc.m_skybox_specular_map;
        let specular_pos_x_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.positive_x_map, 4).unwrap();
        let specular_neg_x_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.negative_x_map, 4).unwrap();
        let specular_pos_y_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.positive_y_map, 4).unwrap();
        let specular_neg_y_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.negative_y_map, 4).unwrap();
        let specular_pos_z_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.positive_z_map, 4).unwrap();
        let specular_neg_z_map = RenderResourceBase::load_texture_hdr(&skybox_specular_map.negative_z_map, 4).unwrap();

        let brdf_map = RenderResourceBase::load_texture_hdr(&level_resource_desc.m_ibl_resource_desc.m_brdf_map, 4).unwrap();

        self.create_ibl_samplers(rhi);

        let irradiance_maps = [
            irradiace_pos_x_map,
            irradiace_neg_x_map,
            irradiace_pos_y_map,
            irradiace_neg_y_map,
            irradiace_pos_z_map,
            irradiace_neg_z_map,
        ];
        let specular_maps = [
            specular_pos_x_map,
            specular_neg_x_map,
            specular_pos_y_map,
            specular_neg_y_map,
            specular_pos_z_map,
            specular_neg_z_map,
        ];

        self.create_ibl_textures(rhi, &irradiance_maps, &specular_maps);

        (
            self.m_global_render_resource.borrow_mut()._ibl_resource._brdf_lut_texture_image,
            self.m_global_render_resource.borrow_mut()._ibl_resource._brdf_lut_texture_image_allocation,
            self.m_global_render_resource.borrow_mut()._ibl_resource._brdf_lut_texture_image_view,
        ) = rhi.create_texture_image(
            brdf_map.m_width,
            brdf_map.m_height,
            &brdf_map.m_pixels,
            brdf_map.m_format,
            0,
        ).unwrap();

        let color_grading_map = RenderResourceBase::load_texture(
            &level_resource_desc.m_color_grading_resource_desc.m_color_grading_map,
            false
        ).unwrap();

        (
            self.m_global_render_resource.borrow_mut()._color_grading_resource._color_grading_lut_texture_image,
            self.m_global_render_resource.borrow_mut()._color_grading_resource._color_grading_lut_texture_image_allocation,
            self.m_global_render_resource.borrow_mut()._color_grading_resource._color_grading_lut_texture_image_view,
        ) = rhi.create_texture_image(
            color_grading_map.m_width,
            color_grading_map.m_height,
            &color_grading_map.m_pixels,
            color_grading_map.m_format,
            0,
        ).unwrap();

    }
    
    pub fn update_per_frame_buffer(&mut self, _render_scene: &RenderScene, camera: &RenderCamera){
        let view_matrix = camera.get_view_matrix();
        let proj_matrix = camera.get_pers_proj_matrix();
        let camera_position = camera.position();
        let proj_view_matrix = proj_matrix * view_matrix;

        self.m_mesh_perframe_storage_buffer_object.proj_view_matrix = proj_view_matrix;
        self.m_mesh_perframe_storage_buffer_object.camera_position = *camera_position;
        
    }

    pub fn upload_game_object_render_resource(&mut self, rhi: &VulkanRHI, render_entity: &RenderEntity, mesh_data: &RenderMeshData, material_data: &RenderMaterialData) {
        self.get_or_create_vulkan_mesh(rhi, render_entity, mesh_data);
        self.get_or_create_vulkan_material(rhi, render_entity, material_data);
    }

    pub fn upload_game_object_render_resource_mesh(&mut self, rhi: &VulkanRHI, render_entity: &RenderEntity, mesh_data: &RenderMeshData){
        self.get_or_create_vulkan_mesh(rhi, render_entity, mesh_data);
    }

    pub fn upload_game_object_render_resource_material(&mut self, rhi: &VulkanRHI, render_entity: &RenderEntity, material_data: &RenderMaterialData){
        self.get_or_create_vulkan_material(rhi, render_entity, material_data);
    }

    pub fn get_entity_mesh(&self, entity: &RenderEntity) -> &Rc<VulkanMesh> {
        self.m_vulkan_meshes.get(&entity.m_mesh_asset_id).unwrap()
    }

    pub fn get_entity_material(&self, entity: &RenderEntity) -> &Rc<VulkanPBRMaterial> {
        self.m_vulkan_pbr_materials.get(&entity.m_material_asset_id).unwrap()
    }
}

impl RenderResource {
    fn get_or_create_vulkan_mesh(&mut self, rhi: &VulkanRHI, entity: &RenderEntity, mesh_data: &RenderMeshData) -> &VulkanMesh {
        let assetid = entity.m_mesh_asset_id;

        if let None = self.m_vulkan_meshes.get(&assetid) {
            let mut now_mesh = VulkanMesh::default();

            let index_buffer_size = mesh_data.m_static_mesh_data.m_index_buffer.m_data.len();
            let index_buffer_data = &mesh_data.m_static_mesh_data.m_index_buffer.m_data;

            let vertex_buffer_data = &mesh_data.m_static_mesh_data.m_vertex_buffer.m_data;

            if mesh_data.m_skeleton_binding_buffer.m_data.len() > 0{
                unimplemented!();
            }
            else{
                let vertex_buffer_data: &[MeshVertexDataDefinition] = bytemuck::cast_slice(&vertex_buffer_data);
                Self::update_mesh_data(
                    rhi,
                    false,
                    index_buffer_size as u32,
                    index_buffer_data,
                    vertex_buffer_data,
                    &mut now_mesh
                ).unwrap();
            }

            self.m_vulkan_meshes.insert(assetid, Rc::new(now_mesh));
        }

        self.m_vulkan_meshes.get(&assetid).unwrap()
    }

    fn get_or_create_vulkan_material(&mut self, rhi: &VulkanRHI, entity: &RenderEntity, material_data: &RenderMaterialData) -> &VulkanPBRMaterial {
        let assetid = entity.m_material_asset_id;

        if let None = self.m_vulkan_pbr_materials.get(&assetid) {
            let mut now_material = VulkanPBRMaterial::default();
            let empty_image = [255, 255, 255, 255];
            let empty_image_data = empty_image.as_slice();

            let mut base_color_image_pixels = empty_image_data;
            let mut base_color_image_width = 1;
            let mut base_color_image_height = 1;
            let mut base_color_image_format = vk::Format::R8G8B8A8_SRGB;
            if let Some(texture) = &material_data.m_base_color_texture {
                base_color_image_pixels = &texture.m_pixels;
                base_color_image_width = texture.m_width;
                base_color_image_height = texture.m_height;
                base_color_image_format = texture.m_format;
            }

            let mut metallic_roughness_image_pixels = empty_image_data;
            let mut metallic_roughness_image_width = 1;
            let mut metallic_roughness_image_height = 1;
            let mut metallic_roughness_image_format = vk::Format::R8G8B8A8_UNORM;
            if let Some(texture) = &material_data.m_metallic_roughness_texture {
                metallic_roughness_image_pixels = &texture.m_pixels;
                metallic_roughness_image_width = texture.m_width;
                metallic_roughness_image_height = texture.m_height;
                metallic_roughness_image_format = texture.m_format;
            }

            let mut normal_roughness_image_pixels = empty_image_data;
            let mut normal_roughness_image_width = 1;
            let mut normal_roughness_image_height = 1;
            let mut normal_roughness_image_format = vk::Format::R8G8B8A8_UNORM;
            if let Some(texture) = &material_data.m_normal_texture {
                normal_roughness_image_pixels = &texture.m_pixels;
                normal_roughness_image_width = texture.m_width;
                normal_roughness_image_height = texture.m_height;
                normal_roughness_image_format = texture.m_format;
            }

            let mut occlusion_image_pixels = empty_image_data;
            let mut occlusion_image_width = 1;
            let mut occlusion_image_height = 1;
            let mut occlusion_image_format = vk::Format::R8G8B8A8_UNORM;
            if let Some(texture) = &material_data.m_occlusion_texture {
                occlusion_image_pixels = &texture.m_pixels;
                occlusion_image_width = texture.m_width;
                occlusion_image_height = texture.m_height;
                occlusion_image_format = texture.m_format;
            }

            let mut emissive_image_pixels = empty_image_data;
            let mut emissive_image_width = 1;
            let mut emissive_image_height = 1;
            let mut emissive_image_format = vk::Format::R8G8B8A8_UNORM;
            if let Some(texture) = &material_data.m_emissive_texture {
                emissive_image_pixels = &texture.m_pixels;
                emissive_image_width = texture.m_width;
                emissive_image_height = texture.m_height;
                emissive_image_format = texture.m_format;
            }

            {
                let buffer_size = std::mem::size_of::<MeshPerMaterialUniformBufferObject>();

                let (staging_buffer, staging_memory) = rhi.create_buffer(
                    buffer_size as u64, 
                    vk::BufferUsageFlags::TRANSFER_SRC, 
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                ).unwrap();

                let staging_buffer_data = rhi.map_memory(
                    staging_memory, 0, buffer_size as u64, vk::MemoryMapFlags::empty()
                ).unwrap();

                let material_uniform_buffer_info = unsafe{
                    &mut *(staging_buffer_data as *mut MeshPerMaterialUniformBufferObject)
                };
                material_uniform_buffer_info.is_blend = entity.m_blend as u32;
                material_uniform_buffer_info.id_double_sided = entity.m_double_sided as u32;
                material_uniform_buffer_info.base_color_factor = entity.m_base_color_factor;
                material_uniform_buffer_info.metallic_factor = entity.m_metallic_factor;
                material_uniform_buffer_info.roughness_factor = entity.m_roughness_factor;
                material_uniform_buffer_info.normal_scale = entity.m_normal_scale;
                material_uniform_buffer_info.occlusion_strength = entity.m_occlusion_strength;
                material_uniform_buffer_info.emissive_factor = entity.m_emissive_factor;

                rhi.unmap_memory(staging_memory);

                (now_material.material_uniform_buffer, now_material.material_uniform_buffer_allocation) = rhi.create_buffer(
                    buffer_size as u64,
                    vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                ).unwrap();//todo: alignment

                rhi.copy_buffer(
                    staging_buffer, now_material.material_uniform_buffer, 0, 0, buffer_size as u64
                ).unwrap();

                rhi.destroy_buffer(staging_buffer);
                rhi.free_memory(staging_memory);
            }

            let mut update_texture_data = TextureDataToUpdate {
                base_color_image_pixels,
                base_color_image_width,
                base_color_image_height,
                base_color_image_format,

                metallic_roughness_image_pixels,
                metallic_roughness_image_width,
                metallic_roughness_image_height,
                metallic_roughness_image_format,

                normal_roughness_image_pixels,
                normal_roughness_image_width,
                normal_roughness_image_height,
                normal_roughness_image_format,

                occlusion_image_pixels,
                occlusion_image_width,
                occlusion_image_height,
                occlusion_image_format,

                emissive_image_pixels,
                emissive_image_width,
                emissive_image_height,
                emissive_image_format,
                
                now_material: &mut now_material,
            };

            Self::update_texture_image_data(rhi, &mut update_texture_data);

            let set_layouts = [self.m_material_descriptor_set_layout];

            let material_descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(rhi.get_descriptor_pool())
                .set_layouts(&set_layouts);
            now_material.material_descriptor_set = rhi.allocate_descriptor_sets(&material_descriptor_set_alloc_info).unwrap()[0];

            let material_uniform_buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(now_material.material_uniform_buffer)
                .offset(0)
                .range(size_of::<MeshPerMaterialUniformBufferObject>() as u64);

            let base_color_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(now_material.base_color_image_view)
                .sampler(rhi.get_or_create_mipmap_sampler(
                    base_color_image_width,
                    base_color_image_height,
                    RHISamplerType::Nearest,
                ).unwrap());
            let metallic_roughness_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(now_material.metallic_roughness_image_view)
                .sampler(rhi.get_or_create_mipmap_sampler(
                    metallic_roughness_image_width,
                    metallic_roughness_image_height,
                    RHISamplerType::Linear,
                ).unwrap());
            let normal_roughness_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(now_material.normal_image_view)
                .sampler(rhi.get_or_create_mipmap_sampler(
                    normal_roughness_image_width,
                    normal_roughness_image_height,
                    RHISamplerType::Linear,
                ).unwrap());
            let occlusion_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(now_material.occlusion_image_view)
                .sampler(rhi.get_or_create_mipmap_sampler(
                    occlusion_image_width,
                    occlusion_image_height,
                    RHISamplerType::Linear,
                ).unwrap());
            let emissive_image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(now_material.emissive_image_view)
                .sampler(rhi.get_or_create_mipmap_sampler(
                    emissive_image_width,
                    emissive_image_height,
                    RHISamplerType::Linear,
                ).unwrap());

            let mesh_descriptor_writes_info = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&[material_uniform_buffer_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[base_color_image_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(2)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[metallic_roughness_image_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(3)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[normal_roughness_image_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(4)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[occlusion_image_info])
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(now_material.material_descriptor_set)
                    .dst_binding(5)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[emissive_image_info])
                    .build(),
            ];

            rhi.update_descriptor_sets(&mesh_descriptor_writes_info).unwrap();

            self.m_vulkan_pbr_materials.insert(assetid, Rc::new(now_material));
        }

        self.m_vulkan_pbr_materials.get(&assetid).unwrap()
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
            let vertex_count = vertex_buffer_data.len();

            let vertex_position_buffer_size = size_of::<VulkanMeshVertexPosition>() * vertex_count;
            let vertex_varying_enable_blending_buffer_size = size_of::<VulkanMeshVertexVaryingEnableBlending>() * vertex_count;
            let vertex_varying_buffer_size = size_of::<VulkanMeshVertexVarying>() * vertex_count;

            let vertex_position_buffer_offset = 0;
            let vertex_varying_enable_blending_buffer_offset = vertex_position_buffer_offset + vertex_position_buffer_size;
            let vertex_varying_buffer_offset = vertex_varying_enable_blending_buffer_offset + vertex_varying_enable_blending_buffer_size;

            let staging_buffer_size = vertex_position_buffer_size + vertex_varying_enable_blending_buffer_size + vertex_varying_buffer_size;

            let (staging_buffer, staging_memory) = rhi.create_buffer(
                staging_buffer_size as u64, 
                vk::BufferUsageFlags::TRANSFER_SRC, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let staging_buffer_data = rhi.map_memory(
                staging_memory, 0, staging_buffer_size as u64, vk::MemoryMapFlags::empty()
            )?;

            let mesh_vertex_positions = unsafe{
                std::slice::from_raw_parts_mut::<VulkanMeshVertexPosition>(
                    staging_buffer_data as *mut VulkanMeshVertexPosition,
                    vertex_count,
                )
            };
            let mesh_vertex_blending_varyings = unsafe{
                std::slice::from_raw_parts_mut::<VulkanMeshVertexVaryingEnableBlending>(
                    (staging_buffer_data as *mut u8).add(vertex_varying_enable_blending_buffer_offset) as *mut VulkanMeshVertexVaryingEnableBlending,
                    vertex_count,
                )
            };
            let mesh_vertex_varyings = unsafe{
                std::slice::from_raw_parts_mut::<VulkanMeshVertexVarying>(
                    (staging_buffer_data as *mut u8).add(vertex_varying_buffer_offset) as *mut VulkanMeshVertexVarying,
                    vertex_count,
                )
            };

            for vertex_index in 0..vertex_buffer_data.len() {
                let normal = Vector3::new(
                    vertex_buffer_data[vertex_index].nx,
                    vertex_buffer_data[vertex_index].ny,
                    vertex_buffer_data[vertex_index].nz,
                );
                let tangent = Vector3::new(
                    vertex_buffer_data[vertex_index].tx,
                    vertex_buffer_data[vertex_index].ty,
                    vertex_buffer_data[vertex_index].tz,
                );
                mesh_vertex_positions[vertex_index].position = Vector3::new(
                    vertex_buffer_data[vertex_index].x,
                    vertex_buffer_data[vertex_index].y,
                    vertex_buffer_data[vertex_index].z,
                );
                mesh_vertex_blending_varyings[vertex_index].normal = normal;
                mesh_vertex_blending_varyings[vertex_index].tangent = tangent;

                mesh_vertex_varyings[vertex_index].texcoord = Vector2::new(
                    vertex_buffer_data[vertex_index].u,
                    vertex_buffer_data[vertex_index].v,
                )
            }
            
            rhi.unmap_memory(staging_memory);
            (now_mesh.mesh_vertex_position_buffer, now_mesh.mesh_vertex_position_buffer_allocation) = rhi.create_buffer(
                vertex_position_buffer_size as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            (now_mesh.mesh_vertex_varying_enable_blending_buffer, now_mesh.mesh_vertex_varying_enable_blending_buffer_allocation) = rhi.create_buffer(
                vertex_varying_enable_blending_buffer_size as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            (now_mesh.mesh_vertex_varying_buffer, now_mesh.mesh_vertex_varying_buffer_allocation) = rhi.create_buffer(
                vertex_varying_buffer_size as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, 
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            rhi.copy_buffer(staging_buffer, now_mesh.mesh_vertex_position_buffer,vertex_position_buffer_offset as u64,0, vertex_position_buffer_size as u64)?;
            rhi.copy_buffer(staging_buffer, now_mesh.mesh_vertex_varying_enable_blending_buffer,vertex_varying_enable_blending_buffer_offset as u64,0, vertex_varying_enable_blending_buffer_size as u64)?;
            rhi.copy_buffer(staging_buffer, now_mesh.mesh_vertex_varying_buffer, vertex_varying_buffer_offset as u64,0, vertex_varying_buffer_size as u64)?;
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

    fn update_texture_image_data(rhi: &VulkanRHI, texture_data: &mut TextureDataToUpdate) {
        (
            texture_data.now_material.base_color_texture_image,
            texture_data.now_material.base_color_image_allocation,
            texture_data.now_material.base_color_image_view
        ) = rhi.create_texture_image(
            texture_data.base_color_image_width,
            texture_data.base_color_image_height,
            texture_data.base_color_image_pixels,
            texture_data.base_color_image_format,
            0,
        ).unwrap();

        (
            texture_data.now_material.metallic_roughness_texture_image,
            texture_data.now_material.metallic_roughness_image_allocation,
            texture_data.now_material.metallic_roughness_image_view
        ) = rhi.create_texture_image(
            texture_data.metallic_roughness_image_width,
            texture_data.metallic_roughness_image_height,
            texture_data.metallic_roughness_image_pixels,
            texture_data.metallic_roughness_image_format,
            0,
        ).unwrap();

        (
            texture_data.now_material.normal_texture_image,
            texture_data.now_material.normal_image_allocation,
            texture_data.now_material.normal_image_view
        ) = rhi.create_texture_image(
            texture_data.normal_roughness_image_width,
            texture_data.normal_roughness_image_height,
            texture_data.normal_roughness_image_pixels,
            texture_data.normal_roughness_image_format,
            0,
        ).unwrap();

        (
            texture_data.now_material.occlusion_texture_image,
            texture_data.now_material.occlusion_image_allocation,
            texture_data.now_material.occlusion_image_view
        ) = rhi.create_texture_image(
            texture_data.occlusion_image_width,
            texture_data.occlusion_image_height,
            texture_data.occlusion_image_pixels,
            texture_data.occlusion_image_format,
            0,
        ).unwrap();

        (
            texture_data.now_material.emissive_texture_image,
            texture_data.now_material.emissive_image_allocation,
            texture_data.now_material.emissive_image_view
        ) = rhi.create_texture_image(
            texture_data.emissive_image_width,
            texture_data.emissive_image_height,
            texture_data.emissive_image_pixels,
            texture_data.emissive_image_format,
            0,
        ).unwrap();
    }
    
    fn create_and_map_storage_buffer(&mut self, rhi: &VulkanRHI){
        let _storage_buffer = &mut self.m_global_render_resource.borrow_mut()._storage_buffer;
        let frames_in_flight = vulkan_rhi::K_MAX_FRAMES_IN_FLIGHT;

        let properties = rhi.get_physical_device_properties();
        _storage_buffer._min_uniform_buffer_offset_alignment = properties.limits.min_uniform_buffer_offset_alignment as u32;
        _storage_buffer._min_storage_buffer_offset_alignment = properties.limits.min_storage_buffer_offset_alignment as u32;
        _storage_buffer._max_storage_buffer_range = properties.limits.max_storage_buffer_range as u32;
        _storage_buffer._non_coherent_atom_size = properties.limits.non_coherent_atom_size as u32;

        let global_storage_buffer_size = 1024 * 1024 * 128;
        (_storage_buffer._global_upload_ringbuffer, _storage_buffer._global_upload_ringbuffer_memory) = rhi.create_buffer(
            global_storage_buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        ).unwrap();

        _storage_buffer._global_upload_ringbuffers_begin.resize(frames_in_flight, 0);
        _storage_buffer._global_upload_ringbuffers_end.resize(frames_in_flight, 0);
        _storage_buffer._global_upload_ringbuffers_size.resize(frames_in_flight, 0);

        for i in 0..frames_in_flight {
            _storage_buffer._global_upload_ringbuffers_begin[i] =
                (global_storage_buffer_size as u32 * i as u32) / frames_in_flight as u32;
            _storage_buffer._global_upload_ringbuffers_size[i] =
                (global_storage_buffer_size as u32 * (i + 1) as u32) / frames_in_flight as u32 - 
                (global_storage_buffer_size as u32 * i as u32) / frames_in_flight as u32;
        }

        _storage_buffer._global_upload_ringbuffer_pointer = rhi.map_memory(
            _storage_buffer._global_upload_ringbuffer_memory, 
            0,
            global_storage_buffer_size as u64, 
            vk::MemoryMapFlags::empty()
        ).unwrap();
    }

    fn create_ibl_samplers(&mut self, rhi: &VulkanRHI) {
        let physical_device_properties = rhi.get_physical_device_properties();
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(true)
            .max_anisotropy(physical_device_properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mip_lod_bias(0.0)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(0.0);

        if self.m_global_render_resource.borrow()._ibl_resource._brdf_lut_texture_sampler != vk::Sampler::null() {
            rhi.destroy_sampler(self.m_global_render_resource.borrow()._ibl_resource._brdf_lut_texture_sampler);
        }

        self.m_global_render_resource.borrow_mut()._ibl_resource._brdf_lut_texture_sampler =
            rhi.create_sampler(&sampler_info).unwrap();

        if self.m_global_render_resource.borrow()._ibl_resource._specular_texture_sampler != vk::Sampler::null() {
            rhi.destroy_sampler(self.m_global_render_resource.borrow()._ibl_resource._specular_texture_sampler);
        }

        self.m_global_render_resource.borrow_mut()._ibl_resource._specular_texture_sampler =
            rhi.create_sampler(&sampler_info).unwrap();
    }

    fn create_ibl_textures(&mut self, rhi: &VulkanRHI, irradiance_maps: &[TextureData; 6], specular_maps: &[TextureData; 6]) {
        let irradiance_cubemap_miplevels =
            (irradiance_maps[0].m_width.max(irradiance_maps[0].m_height) as f32).log2().floor() as u32 + 1;

        (self.m_global_render_resource.borrow_mut()._ibl_resource._irradiance_texture_image,
        self.m_global_render_resource.borrow_mut()._ibl_resource._irradiance_texture_image_allocation,
        self.m_global_render_resource.borrow_mut()._ibl_resource._irradiance_texture_image_view) = rhi.create_cube_map(
            irradiance_maps[0].m_width, 
            irradiance_maps[0].m_height, 
            &irradiance_maps.iter().map(|texture|texture.m_pixels.as_slice()).collect_array().unwrap(), 
            irradiance_maps[0].m_format, 
            irradiance_cubemap_miplevels
        ).unwrap();

        let specular_cubemap_miplevels =
            (specular_maps[0].m_width.max(specular_maps[0].m_height) as f32).log2().floor() as u32 + 1;
        
        (self.m_global_render_resource.borrow_mut()._ibl_resource._specular_texture_image,
        self.m_global_render_resource.borrow_mut()._ibl_resource._specular_texture_image_allocation,
        self.m_global_render_resource.borrow_mut()._ibl_resource._specular_texture_image_view) = rhi.create_cube_map(
            specular_maps[0].m_width, 
            specular_maps[0].m_height, 
            &specular_maps.iter().map(|texture|texture.m_pixels.as_slice()).collect_array().unwrap(), 
            specular_maps[0].m_format, 
            specular_cubemap_miplevels
        ).unwrap();
    }
}