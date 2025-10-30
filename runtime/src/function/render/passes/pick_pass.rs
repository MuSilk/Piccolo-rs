use std::{cell::RefCell, os::raw::c_void, rc::Rc};

use crate::{core::math::vector2::Vector2, function::render::{interface::vulkan::vulkan_rhi::{VulkanRHI, VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC}, render_common::{MeshInefficientPickPerdrawcallStorageBufferObject, MeshInefficientPickPerdrawcallVertexBlendingStorageBufferObject, MeshInefficientPickPerframeStorageBufferObject}, render_helper::round_up, render_mesh::MeshVertex, render_pass::{RenderPass, RenderPipelineBase}, render_resource::RenderResource}, shader::generated::shader::{MESH_INEFFICIENT_PICK_FRAG, MESH_INEFFICIENT_PICK_VERT}};

use anyhow::Result;
use linkme::distributed_slice;
use vulkanalia::{prelude::v1_0::*};

pub struct PickPassInitInfo<'a> {
    pub per_mesh_layout: vk::DescriptorSetLayout,
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
}


#[derive(Default)]
pub struct PickPass{
    pub m_render_pass: RenderPass,
    m_per_mesh_layout: vk::DescriptorSetLayout,
    m_mesh_inefficient_pick_perframe_storage_buffer_object: MeshInefficientPickPerframeStorageBufferObject,
}

#[distributed_slice(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC)]
static STORAGE_BUFFER_DYNAMIC_COUNT: u32 = 3;

impl PickPass {
    pub fn initialize(&mut self, info: &PickPassInitInfo) -> Result<()> {
        self.m_render_pass.initialize();
        let rhi = info.rhi.borrow();
        self.m_per_mesh_layout = info.per_mesh_layout;

        self.setup_attachments(&rhi)?;
        self.setup_render_pass(&rhi)?;
        self.setup_framebuffer(&rhi)?;
        self.setup_descriptor_layout(&rhi)?;
        self.setup_pipelines(&rhi)?;
        self.setup_descriptor_set(&rhi)?;

        Ok(())
    }

    pub fn prepare_pass_data(&mut self, render_resource: &RenderResource) {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        self.m_mesh_inefficient_pick_perframe_storage_buffer_object.rt_width = swapchain_info.extent.width;
        self.m_mesh_inefficient_pick_perframe_storage_buffer_object.rt_height = swapchain_info.extent.height;
        self.m_mesh_inefficient_pick_perframe_storage_buffer_object.proj_view_matrix =
            render_resource.m_mesh_inefficient_pick_perframe_storage_buffer_object.proj_view_matrix.clone();
    }

    pub fn recreate_framebuffer(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_framebuffer.attachments.iter().for_each(|attachment| {
            rhi.destroy_image(attachment.image);
            rhi.destroy_image_view(attachment.view);
            rhi.free_memory(attachment.mem);
        });
        rhi.destroy_framebuffer(self.m_render_pass.m_framebuffer.framebuffer);

        self.setup_attachments(rhi)?;
        self.setup_render_pass(rhi)?;
        Ok(())
    }

    pub fn pick(&self, picked_uv: &Vector2) -> u32 {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();

        let picked_pixel_index = {
            let rhi = rhi.borrow();
            let swapchain_info = rhi.get_swapchain_info();
            let pixel_x = (picked_uv.x * swapchain_info.viewport.width +  swapchain_info.viewport.x) as i32;
            let pixel_y = (picked_uv.y * swapchain_info.viewport.height + swapchain_info.viewport.y) as i32;
            if pixel_x >= swapchain_info.extent.width as i32 || pixel_y >= swapchain_info.extent.height as i32 {
                return 0;
            }
            swapchain_info.extent.width as i32 * pixel_y + pixel_x as i32
        };
        {
            rhi.borrow_mut().prepare_context();
        }
        {
            let resource = self.m_render_pass.m_global_render_resource.upgrade().unwrap();
            let mut resource = resource.borrow_mut();
            let current_frame_index = rhi.borrow().get_current_frame_index();
            resource._storage_buffer._global_upload_ringbuffers_end[current_frame_index] =
                resource._storage_buffer._global_upload_ringbuffers_begin[current_frame_index];
        }
        {
            let rhi = rhi.borrow();
            rhi.wait_for_fence().unwrap();
            rhi.reset_command_pool().unwrap();
        }

        {
            let rhi = rhi.borrow();
            rhi.prepare_frame().unwrap();
            let command_buffer = rhi.get_current_command_buffer();

            let transfer_to_render_barrier = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_queue_family_index(rhi.get_queue_family_indices().graphics_family.unwrap())
                .dst_queue_family_index(rhi.get_queue_family_indices().graphics_family.unwrap())
                .image(self.m_render_pass.m_framebuffer.attachments[0].image)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build()
                )
                .build();
            
            rhi.cmd_pipeline_barrier(
                rhi.get_current_command_buffer(), 
                vk::PipelineStageFlags::ALL_COMMANDS, 
                vk::PipelineStageFlags::ALL_COMMANDS, 
                vk::DependencyFlags::empty(), 
                &[], 
                &[], 
                &[transfer_to_render_barrier]
            );

            let swapchain_info = rhi.get_swapchain_info();

            rhi.cmd_set_viewport(command_buffer, 0, &[*swapchain_info.viewport]);
            rhi.cmd_set_scissor(command_buffer, 0, &[*swapchain_info.scissor]);

            let mut  clear_values = [vk::ClearValue::default(); 2];
            clear_values[0].color.uint32 = [0, 0, 0, 0];
            clear_values[1].depth_stencil.depth = 1.0;
            clear_values[1].depth_stencil.stencil = 0;

            let begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.m_render_pass.m_framebuffer.render_pass)
                .framebuffer(self.m_render_pass.m_framebuffer.framebuffer)
                .render_area(vk::Rect2D::builder()
                    .offset(vk::Offset2D { x: 0, y: 0 })
                    .extent(swapchain_info.extent)
                    .build()
                )
                .clear_values(&clear_values)
                .build();

            rhi.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::INLINE);

            rhi.push_event(command_buffer, "Mesh Inefficient Pick\0", [1.0; 4]);
            rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.m_render_pass.m_render_pipeline[0].pipeline);
            
            rhi.cmd_set_viewport(command_buffer, 0, &[*swapchain_info.viewport]);
            rhi.cmd_set_scissor(command_buffer, 0, &[*swapchain_info.scissor]);
            
            let render_resource = self.m_render_pass.m_global_render_resource.upgrade().unwrap();

            let perframe_dynamic_offset = round_up(
                render_resource.borrow()._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()],
                render_resource.borrow()._storage_buffer._min_storage_buffer_offset_alignment
            );

            render_resource.borrow_mut()
                ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()] = 
                    perframe_dynamic_offset + std::mem::size_of::<MeshInefficientPickPerframeStorageBufferObject>() as u32;
            unsafe{
                std::ptr::copy_nonoverlapping(
                    &self.m_mesh_inefficient_pick_perframe_storage_buffer_object as *const _ as *const c_void,
                    render_resource.borrow()._storage_buffer._global_upload_ringbuffer_pointer.add(perframe_dynamic_offset as usize), 
                    std::mem::size_of::<MeshInefficientPickPerframeStorageBufferObject>()
                );
            }

            let m_visible_nodes = RenderPass::m_visible_nodes().borrow();

            for render_mesh_node in m_visible_nodes.p_main_camera_visible_mesh_nodes.upgrade().unwrap().borrow().iter() {
                let mut object = MeshInefficientPickPerdrawcallStorageBufferObject::default();
                object.model_matrix[0] = *render_mesh_node.model_matrix;
                object.node_ids[0] = render_mesh_node.node_id;
                object.enable_vertex_blending[0] = 0;

                let perdrawcall_dynamic_offset = round_up(
                    render_resource.borrow()._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()], 
                    render_resource.borrow()._storage_buffer._min_storage_buffer_offset_alignment
                );
                render_resource.borrow_mut()
                    ._storage_buffer._global_upload_ringbuffers_end[rhi.get_current_frame_index()] = 
                        perdrawcall_dynamic_offset + std::mem::size_of::<MeshInefficientPickPerdrawcallStorageBufferObject>() as u32;

                unsafe{
                    std::ptr::copy_nonoverlapping(
                        &object as *const _ as *const c_void,
                        render_resource.borrow()._storage_buffer._global_upload_ringbuffer_pointer.add(perdrawcall_dynamic_offset as usize), 
                        std::mem::size_of::<MeshInefficientPickPerdrawcallStorageBufferObject>()
                    );
                }

                rhi.cmd_bind_descriptor_sets(
                    command_buffer, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    self.m_render_pass.m_render_pipeline[0].layout,
                    0,
                    &[
                        self.m_render_pass.m_descriptor_infos[0].descriptor_set,
                        render_mesh_node.ref_mesh.upgrade().unwrap().mesh_vertex_blending_descriptor_set,
                    ],
                    &[perframe_dynamic_offset, perdrawcall_dynamic_offset, 0],
                );

                let buffers = [
                    render_mesh_node.ref_mesh.upgrade().unwrap().mesh_vertex_position_buffer,
                ];
                
                rhi.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &[0]);
                rhi.cmd_bind_index_buffer(command_buffer, render_mesh_node.ref_mesh.upgrade().unwrap().mesh_index_buffer, 0, vk::IndexType::UINT16);
                rhi.cmd_draw_indexed(
                    command_buffer, 
                    render_mesh_node.ref_mesh.upgrade().unwrap().mesh_index_count, 
                    1, 
                    0, 
                    0, 
                    0
                );
            }

            //todo: render 
            
            rhi.pop_event(command_buffer);
            rhi.cmd_end_render_pass(command_buffer);
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_frame().unwrap();
            rhi.wait_idle().unwrap();
        }
        {
            let rhi = rhi.borrow();
            let command_buffer = rhi.begin_single_time_commands().unwrap();
            let swapchain_info = rhi.get_swapchain_info();

            let region = vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build()
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: swapchain_info.extent.width,
                    height: swapchain_info.extent.height,
                    depth: 1,
                })
                .build();

            let buffer_size = rhi.get_swapchain_info().extent.width * swapchain_info.extent.height * 4;
            let (staging_buffer, staging_buffer_mem) = rhi.create_buffer(
                buffer_size as u64,
                vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            ).unwrap();

            let copy_to_buffer_barrier = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .src_queue_family_index(rhi.get_queue_family_indices().graphics_family.unwrap())
                .dst_queue_family_index(rhi.get_queue_family_indices().graphics_family.unwrap())
                .image(self.m_render_pass.m_framebuffer.attachments[0].image)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build()
                )
                .build();

            rhi.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[copy_to_buffer_barrier],
            );

            rhi.cmd_copy_image_to_buffer(
                command_buffer, 
                self.m_render_pass.m_framebuffer.attachments[0].image, 
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 
                staging_buffer, 
                &[region]
            );

            rhi.end_single_time_commands(command_buffer).unwrap();

            let data = rhi.map_memory(
                staging_buffer_mem, 
                0, 
                buffer_size as u64, 
                vk::MemoryMapFlags::empty()).unwrap() as *const u32;
            
            let node_id = unsafe {
                *data.add(picked_pixel_index as usize)
            };
            rhi.unmap_memory(staging_buffer_mem);
            rhi.destroy_buffer(staging_buffer);
            rhi.free_memory(staging_buffer_mem);

            return node_id;
        }
    }
}

impl PickPass {
    fn setup_attachments(&mut self, rhi: &VulkanRHI) -> Result<()> {

        let swapchain_info = rhi.get_swapchain_info();

        self.m_render_pass.m_framebuffer.attachments.resize_with(2, Default::default);

        self.m_render_pass.m_framebuffer.attachments[0].format = vk::Format::R32_UINT;
        (
            self.m_render_pass.m_framebuffer.attachments[0].image,
            self.m_render_pass.m_framebuffer.attachments[0].mem,
        ) = rhi.create_image(
            swapchain_info.extent.width,
            swapchain_info.extent.height,
            self.m_render_pass.m_framebuffer.attachments[0].format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageCreateFlags::empty(),
            1, 1
        )?;

        self.m_render_pass.m_framebuffer.attachments[0].view = rhi.create_image_view(
            self.m_render_pass.m_framebuffer.attachments[0].image,
            self.m_render_pass.m_framebuffer.attachments[0].format,
            vk::ImageAspectFlags::COLOR,
            vk::ImageViewType::_2D, 1, 1
        )?;

        Ok(())
    }

    fn setup_render_pass(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let attachements = [
            vk::AttachmentDescription::builder()
                .format(self.m_render_pass.m_framebuffer.attachments[0].format)
                .samples(vk::SampleCountFlags::_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(rhi.get_depth_image_info().format)
                .samples(vk::SampleCountFlags::_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        let color_attachment_refs = [
            vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();
        let subpasses = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .build(),
        ];
        let dependencies: [vk::SubpassDependency; 0] = [];

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachements)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        self.m_render_pass.m_framebuffer.render_pass = rhi.create_render_pass(&create_info)?;

        Ok(())
    }

    fn setup_framebuffer(&mut self, rhi: &VulkanRHI) -> Result<()> {

        let attachments = [
            self.m_render_pass.m_framebuffer.attachments[0].view,
            *rhi.get_depth_image_info().image_view
        ];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .attachments(&attachments)
            .width(rhi.get_swapchain_info().extent.width)
            .height(rhi.get_swapchain_info().extent.height)
            .layers(1)
            .build();

        self.m_render_pass.m_framebuffer.framebuffer = rhi.create_framebuffer(&framebuffer_create_info)?;

        Ok(())
    }

    fn setup_descriptor_layout(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_descriptor_infos.resize_with(1, Default::default);
        let layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
        ];

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings);

        self.m_render_pass.m_descriptor_infos[0].layout = rhi.create_descriptor_set_layout(&layout_create_info)?;

        Ok(())
    }

    fn setup_pipelines(&mut self, rhi: &VulkanRHI) -> Result<()> {

        self.m_render_pass.m_render_pipeline.resize_with(1, Default::default);

        let vert_shader_module = rhi.create_shader_module(&MESH_INEFFICIENT_PICK_VERT)?;
        let frag_shader_module = rhi.create_shader_module(&MESH_INEFFICIENT_PICK_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding_descriptions = &MeshVertex::get_binding_descriptions()[0..1];
        let attribute_descriptions = &MeshVertex::get_attribute_descriptions()[0..1];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let depth_stencil_state: vk::PipelineDepthStencilStateCreateInfoBuilder = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .stencil_test_enable(false);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::all())
                .blend_enable(false)
                .build(),
        ];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let set_layouts = &[
            self.m_render_pass.m_descriptor_infos[0].layout,
            self.m_per_mesh_layout, 
        ];
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts);

        let pipeline_layout = rhi.create_pipeline_layout(&layout_info)?;

        let stages = &[vert_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .dynamic_state(&dynamic_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .subpass(0)
            .build();

        let pipeline = rhi.create_graphics_pipelines(vk::PipelineCache::null(), &[info])?[0];

        rhi.destroy_shader_module(vert_shader_module);
        rhi.destroy_shader_module(frag_shader_module);

        self.m_render_pass.m_render_pipeline[0] = RenderPipelineBase{
            layout: pipeline_layout,
            pipeline,
        };
        Ok(())
    }

    fn setup_descriptor_set(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let set_layouts = [self.m_render_pass.m_descriptor_infos[0].layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(rhi.get_descriptor_pool())
            .set_layouts(&set_layouts);
        
        self.m_render_pass.m_descriptor_infos[0].descriptor_set = rhi.allocate_descriptor_sets(&alloc_info)?[0];

        let render_resource = self.m_render_pass.m_global_render_resource.upgrade().unwrap();

        let perframe_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
            .offset(0)
            .range(std::mem::size_of::<MeshInefficientPickPerframeStorageBufferObject>() as u64);

        let perdrawcall_storage_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
            .offset(0)
            .range(std::mem::size_of::<MeshInefficientPickPerdrawcallStorageBufferObject>() as u64);
        
        let perdrawcall_vertex_blending_storage_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
            .offset(0)
            .range(std::mem::size_of::<MeshInefficientPickPerdrawcallVertexBlendingStorageBufferObject>() as u64);

        let write_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&[perframe_buffer_info])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&[perdrawcall_storage_buffer_info])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&[perdrawcall_vertex_blending_storage_buffer_info])
                .build(),
        ];

        rhi.update_descriptor_sets(&write_info)?;

        Ok(())
    }
}