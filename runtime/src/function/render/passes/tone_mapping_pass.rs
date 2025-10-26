use std::{cell::RefCell, rc::Rc};
use anyhow::Result;
use linkme::distributed_slice;
use vulkanalia::{prelude::v1_0::*, vk::{VertexInputAttributeDescription, VertexInputBindingDescription}};

use crate::{function::render::{interface::vulkan::vulkan_rhi::{VulkanRHI, VULKAN_RHI_DESCRIPTOR_INPUT_ATTACHMENT}, render_pass::{Descriptor, RenderPass, RenderPipelineBase, _MAIN_CAMERA_SUBPASS_TONE_MAPPING}, render_type::RHISamplerType}, shader::generated::shader::{POST_PROCESS_VERT, TONE_MAPPING_FRAG}};

pub struct ToneMappingInitInfo<'a>{
    pub render_pass: vk::RenderPass,
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
    pub input_attachment: vk::ImageView,
}

#[derive(Default)]
pub struct ToneMappingPass {
    pub m_render_pass: RenderPass,
}

impl ToneMappingPass {
    pub fn initialize(&mut self, info: &ToneMappingInitInfo) -> Result<()> {
        self.m_render_pass.initialize();
        self.m_render_pass.m_framebuffer.render_pass = info.render_pass;
        self.setup_descriptor_layout(&info.rhi.borrow())?;
        self.setup_pipelines(&info.rhi.borrow())?;
        self.setup_descriptor_set(&info.rhi.borrow())?;
        self.update_after_framebuffer_recreate(&info.rhi.borrow(), info.input_attachment)?;
        Ok(())
    }
    pub fn draw(&self) {
        let color = [1.0;4];
        let rhi = self.m_render_pass.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "Tone Map", color);
        let info = rhi.get_swapchain_info();
        rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.m_render_pass.m_render_pipeline[0].pipeline);
        rhi.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(info.scissor));
        rhi.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS, 
            self.m_render_pass.m_render_pipeline[0].layout,
            0,
            &[self.m_render_pass.m_descriptor_infos[0].descriptor_set],
            &[],
        );
        rhi.cmd_draw(command_buffer, 3, 1, 0, 0);
        rhi.pop_event(command_buffer);
    }
    pub fn update_after_framebuffer_recreate(&mut self, rhi: &VulkanRHI, input_attachment: vk::ImageView) -> Result<()> {
        let post_process_per_frame_input_attachment_info = vk::DescriptorImageInfo::builder()
            .sampler(*rhi.get_or_create_default_sampler(RHISamplerType::Nearest)?)
            .image_view(input_attachment)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let post_process_descriptor_writes_info = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.m_render_pass.m_descriptor_infos[0].descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&[post_process_per_frame_input_attachment_info])
                .build(),
        ];
        rhi.update_descriptor_sets(&post_process_descriptor_writes_info)?;
        Ok(())
    }
}

#[distributed_slice(VULKAN_RHI_DESCRIPTOR_INPUT_ATTACHMENT)]
static INPUT_ATTACHMENT_COUNT: u32 = 1;

impl ToneMappingPass {
    fn setup_descriptor_layout(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_descriptor_infos.clear();
        let post_process_global_layout_in_color = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let post_process_global_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&post_process_global_layout_in_color);
        self.m_render_pass.m_descriptor_infos.push(Descriptor {
            layout: rhi.create_descriptor_set_layout(&post_process_global_layout_create_info)?,
            descriptor_set: Default::default(),
        });
        Ok(())
    }

    fn setup_pipelines(&mut self, rhi: &VulkanRHI) -> Result<()> {
        self.m_render_pass.m_render_pipeline.clear();
        let vert_shader_module = rhi.create_shader_module(&POST_PROCESS_VERT)?;
        let frag_shader_module = rhi.create_shader_module(&TONE_MAPPING_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[] as &[VertexInputBindingDescription])
            .vertex_attribute_descriptions(&[] as &[VertexInputAttributeDescription]);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
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

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(false);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let set_layouts = &[self.m_render_pass.m_descriptor_infos[0].layout];
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
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(self.m_render_pass.m_framebuffer.render_pass)
            .subpass(_MAIN_CAMERA_SUBPASS_TONE_MAPPING)
            .build();

        let pipeline = rhi.create_graphics_pipelines(vk::PipelineCache::null(), &[info])?[0];

        rhi.destroy_shader_module(vert_shader_module);
        rhi.destroy_shader_module(frag_shader_module);

        self.m_render_pass.m_render_pipeline.push(RenderPipelineBase{
            layout: pipeline_layout,
            pipeline,
        });

        Ok(())
    }

    fn setup_descriptor_set(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let set_layouts = [self.m_render_pass.m_descriptor_infos[0].layout];
        let post_process_global_descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(rhi.get_descriptor_pool())
            .set_layouts(&set_layouts);

        self.m_render_pass.m_descriptor_infos[0].descriptor_set = rhi.allocate_descriptor_sets(&post_process_global_descriptor_set_alloc_info)?[0];
        Ok(())
    }
}