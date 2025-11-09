use std::{cell::RefCell, rc::Rc};

use crate::{function::render::{interface::vulkan::vulkan_rhi::{VulkanRHI, VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC}, render_common::{MeshPointLightShadowPerdrawcallStorageBufferObject, MeshPointLightShadowPerdrawcallVertexBlendingStorageBufferObject, MeshPointLightShadowPerframeStorageBufferObject, S_MAX_POINT_LIGHT_COUNT, S_POINT_LIGHT_SHADOW_MAP_DIMENSION}, render_mesh::MeshVertex, render_pass::{RenderPass, RenderPipelineBase}, render_resource::RenderResource}, shader::generated::shader::{MESH_POINT_LIGHT_SHADOW_FRAG, MESH_POINT_LIGHT_SHADOW_GEOM, MESH_POINT_LIGHT_SHADOW_VERT}};

use anyhow::Result;
use linkme::distributed_slice;
use vulkanalia::{prelude::v1_0::*};

pub struct PointLightShadowPassInitInfo<'a> {
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
}


#[derive(Default)]
pub struct PointLightShadowPass{
    pub m_render_pass: RenderPass,
    m_per_mesh_layout: vk::DescriptorSetLayout,
    m_mesh_point_light_shadow_perframe_storage_buffer_object: MeshPointLightShadowPerframeStorageBufferObject,
}

#[distributed_slice(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC)]
static STORAGE_BUFFER_DYNAMIC_COUNT: u32 = 3;

impl PointLightShadowPass {
    pub fn initialize(&mut self, info: &PointLightShadowPassInitInfo) -> Result<()> {
        self.m_render_pass.initialize();
        let rhi = info.rhi.borrow();

        self.setup_attachments(&rhi)?;
        self.setup_render_pass(&rhi)?;
        self.setup_framebuffer(&rhi)?;
        self.setup_descriptor_layout(&rhi)?;

        Ok(())
    }

    pub fn post_initialize(&mut self, info: &PointLightShadowPassInitInfo) -> Result<()> {
        let rhi = info.rhi.borrow();
        self.setup_pipelines(&rhi)?;
        self.setup_descriptor_set(&rhi)?;
        Ok(())
    }

    pub fn prepare_pass_data(&mut self, render_resource: &RenderResource) {
        self.m_mesh_point_light_shadow_perframe_storage_buffer_object = 
            render_resource.m_mesh_point_light_shadow_perframe_storage_buffer_object.clone();
    }

    pub fn draw(&self) {
        self.draw_model();
    }

    pub fn set_per_mesh_layout(&mut self, layout: vk::DescriptorSetLayout) {
        self.m_per_mesh_layout = layout;
    }
}

impl PointLightShadowPass {
    fn setup_attachments(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_framebuffer.attachments.resize_with(2, Default::default);

        self.m_render_pass.m_framebuffer.attachments[0].format = vk::Format::R32_SFLOAT;
        (
            self.m_render_pass.m_framebuffer.attachments[0].image,
            self.m_render_pass.m_framebuffer.attachments[0].mem,
        ) = rhi.create_image(
            S_POINT_LIGHT_SHADOW_MAP_DIMENSION,
            S_POINT_LIGHT_SHADOW_MAP_DIMENSION,
            self.m_render_pass.m_framebuffer.attachments[0].format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageCreateFlags::empty(),
            2 * S_MAX_POINT_LIGHT_COUNT as u32, 1
        )?;

        self.m_render_pass.m_framebuffer.attachments[0].view = rhi.create_image_view(
            self.m_render_pass.m_framebuffer.attachments[0].image,
            self.m_render_pass.m_framebuffer.attachments[0].format,
            vk::ImageAspectFlags::COLOR,
            vk::ImageViewType::_2D_ARRAY, 2 * S_MAX_POINT_LIGHT_COUNT as u32, 1
        )?;

        self.m_render_pass.m_framebuffer.attachments[1].format = rhi.get_depth_image_info().format;
        (
            self.m_render_pass.m_framebuffer.attachments[1].image,
            self.m_render_pass.m_framebuffer.attachments[1].mem,
        ) = rhi.create_image(
            S_POINT_LIGHT_SHADOW_MAP_DIMENSION,
            S_POINT_LIGHT_SHADOW_MAP_DIMENSION,
            self.m_render_pass.m_framebuffer.attachments[1].format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageCreateFlags::empty(),
            2 * S_MAX_POINT_LIGHT_COUNT as u32, 1
        )?;

        self.m_render_pass.m_framebuffer.attachments[1].view = rhi.create_image_view(
            self.m_render_pass.m_framebuffer.attachments[1].image,
            self.m_render_pass.m_framebuffer.attachments[1].format,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageViewType::_2D_ARRAY, 
            2 * S_MAX_POINT_LIGHT_COUNT as u32, 1
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
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(self.m_render_pass.m_framebuffer.attachments[1].format)
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
        let dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .build(),
        ];

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
            self.m_render_pass.m_framebuffer.attachments[1].view,
        ];

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .attachments(&attachments)
            .width(S_POINT_LIGHT_SHADOW_MAP_DIMENSION)
            .height(S_POINT_LIGHT_SHADOW_MAP_DIMENSION)
            .layers(2 * S_MAX_POINT_LIGHT_COUNT as u32)
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
                .stage_flags(vk::ShaderStageFlags::GEOMETRY | vk::ShaderStageFlags::FRAGMENT)
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

        let vert_shader_module = rhi.create_shader_module(&MESH_POINT_LIGHT_SHADOW_VERT)?;
        let geom_shader_module = rhi.create_shader_module(&MESH_POINT_LIGHT_SHADOW_GEOM)?;
        let frag_shader_module = rhi.create_shader_module(&MESH_POINT_LIGHT_SHADOW_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let geom_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::GEOMETRY)
            .module(geom_shader_module)
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

        let viewports = [vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(S_POINT_LIGHT_SHADOW_MAP_DIMENSION as f32)
            .height(S_POINT_LIGHT_SHADOW_MAP_DIMENSION as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk::Extent2D { 
                width: S_POINT_LIGHT_SHADOW_MAP_DIMENSION, 
                height: S_POINT_LIGHT_SHADOW_MAP_DIMENSION 
            })];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

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

        let stages = &[vert_stage, geom_stage, frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
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

        let perframe_buffer_info = [
            vk::DescriptorBufferInfo::builder()
                .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
                .offset(0)
                .range(std::mem::size_of::<MeshPointLightShadowPerframeStorageBufferObject>() as u64)
                .build()
        ];

        let perdrawcall_storage_buffer_info = [
            vk::DescriptorBufferInfo::builder()
                .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
                .offset(0)
                .range(std::mem::size_of::<MeshPointLightShadowPerdrawcallStorageBufferObject>() as u64)
                .build()
        ];
        
        let perdrawcall_vertex_blending_storage_buffer_info = [
            vk::DescriptorBufferInfo::builder()
                .buffer(render_resource.borrow()._storage_buffer._global_upload_ringbuffer)
                .offset(0)
                .range(std::mem::size_of::<MeshPointLightShadowPerdrawcallVertexBlendingStorageBufferObject>() as u64)
                .build()
        ];

        let write_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&perframe_buffer_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&perdrawcall_storage_buffer_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .buffer_info(&perdrawcall_vertex_blending_storage_buffer_info)
                .build(),
        ];

        rhi.update_descriptor_sets(&write_info)?;

        Ok(())
    }

    fn draw_model(&self) {
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();

        let mut clear_values: [vk::ClearValue; 2] = [Default::default(); 2];
        clear_values[0].color.float32 = [1.0,0.0,0.0,0.0];
        clear_values[1].depth_stencil = vk::ClearDepthStencilValue{
            depth: 1.0,
            stencil: 0,
        };

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .framebuffer(self.m_render_pass.m_framebuffer.framebuffer)
            .render_area(vk::Rect2D::builder()
                .offset(vk::Offset2D{x: 0, y: 0})
                .extent(vk::Extent2D{
                    width: S_POINT_LIGHT_SHADOW_MAP_DIMENSION, 
                    height: S_POINT_LIGHT_SHADOW_MAP_DIMENSION
                })
                .build())
            .clear_values(&clear_values);

        rhi.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::INLINE);
        rhi.push_event(command_buffer, "Point Light Shadow\0", [1.0;4]);

        rhi.pop_event(command_buffer);
        rhi.cmd_end_render_pass(command_buffer);
    }
}