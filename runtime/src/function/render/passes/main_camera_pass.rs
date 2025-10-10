

use std::{cell::RefCell, rc::Rc, slice};

use crate::{function::render::{interface::vulkan::vulkan_rhi::VulkanRHI, render_mesh::MeshVertex, render_pass::{Descriptor, FrameBufferAttachment, Framebuffer, RenderPass, RenderPipelineBase, _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD, _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_DEPTH, _MAIN_CAMERA_PASS_GBUFFER_A, _MAIN_CAMERA_PASS_GBUFFER_B, _MAIN_CAMERA_PASS_GBUFFER_C, _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD, _MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE}, render_pass_base::{RenderPassBase, RenderPassCommonInfo, RenderPassCreateInfo}}, shader::generated::shader::{MESH_GBUFFER_FRAG, MESH_VERT}};

use anyhow::Result;
use vulkanalia::{prelude::v1_0::*};

pub struct MainCameraPassCreateInfo<'a> {
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
}

enum LayoutType {
    PerMesh,
    MeshGlobal,
    SkyBox,
    Axis,
    Partical,
    DeferredLighting,
    EnumCount,
}

enum RenderPipelineType {
    MeshGBuffer,
    DeferredLighting,
    MeshLighting,
    SkyBox,
    Axis,
    Partical,
    EnumCount,
}

#[derive(Default)]
pub struct MainCameraPass{
    m_base: RenderPass,
    m_swapchain_framebuffers: Vec<vk::Framebuffer>,
}

impl MainCameraPass {
    pub fn create(info: &MainCameraPassCreateInfo) -> Result<Self> {
        let mut camera_render_pass = MainCameraPass::default();
        camera_render_pass.m_base.set_common_info(&RenderPassCommonInfo{
            rhi: info.rhi,
        });
        let rhi = info.rhi.borrow();
        camera_render_pass.setup_attachments();
        camera_render_pass.setup_render_pass(&rhi)?;
        camera_render_pass.setup_framebuffer(&rhi)?;
        camera_render_pass.setup_descriptor_layout(&rhi)?;
        camera_render_pass.setup_pipelines(&rhi)?;
        Ok(camera_render_pass)
    }

    pub fn recreate_after_swapchain(&mut self, rhi: &VulkanRHI) -> Result<()>{
        for framebuffer in self.m_swapchain_framebuffers.drain(..){
            rhi.destroy_framebuffer(framebuffer);
        }
        self.setup_framebuffer(rhi)?;
        Ok(())
    }

    pub fn destroy(&self) {
        let rhi = self.m_base.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        self.m_swapchain_framebuffers.iter().for_each(|f| rhi.destroy_framebuffer(*f));
        rhi.destroy_pipeline(self.m_base.m_render_pipeline[0].pipeline);
        rhi.destroy_pipeline_layout(self.m_base.m_render_pipeline[0].layout);
        // rhi.destroy_descriptor_set_layout(self.m_base.m_descriptor_infos[0].layout);
        rhi.destroy_render_pass(self.m_base.m_framebuffer.render_pass);
    }

//     pub fn get_pipeline(&self) -> &DebugDrawPipelineBase {
//         &self.m_render_pipelines[0]
//     }

//     pub fn get_framebuffer(&self) -> &DebugDrawFramebuffer {
//         &self.m_framebuffer
//     }

    pub fn draw(&self, current_swapchain_image_index: usize) -> Result<()> {
        let color = [1.0;4];
        let rhi = self.m_base.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let command_buffer = rhi.get_current_command_buffer();
        rhi.push_event(command_buffer, "maincamerabasepass", color);
        let info = rhi.get_swapchain_info();
        rhi.cmd_set_viewport(command_buffer, 0, slice::from_ref(info.viewport));
        rhi.cmd_set_scissor(command_buffer, 0, slice::from_ref(info.scissor));

        self.draw_object(current_swapchain_image_index)?;

        rhi.pop_event(command_buffer);
        Ok(())
    }
}

impl MainCameraPass {
    fn setup_attachments(&mut self) {
        
    }

    fn setup_render_pass(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(rhi.get_swapchain_info().image_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(rhi.get_depth_image_info().format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];
        
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .depth_stencil_attachment(&depth_stencil_attachment_ref);

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | 
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT |
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = &[color_attachment, depth_stencil_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);

        self.m_base.m_framebuffer.render_pass = rhi.create_render_pass(&info)?;

        Ok(())
    }

    fn setup_framebuffer(&mut self, rhi: &VulkanRHI) -> Result<()> {
        let swapchain_info = rhi.get_swapchain_info();
        let depth_image_info = rhi.get_depth_image_info();
        let framebuffers =  swapchain_info.image_views
            .iter()
            .map(|i| {
                let attachments = &[*i, *depth_image_info.image_view];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(self.m_base.m_framebuffer.render_pass)
                    .attachments(attachments)
                    .width(swapchain_info.extent.width)
                    .height(swapchain_info.extent.height)
                    .layers(1);

                rhi.create_framebuffer(&create_info)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.m_swapchain_framebuffers = framebuffers;
        Ok(())
    }

    fn setup_descriptor_layout(&mut self, rhi: &VulkanRHI) -> Result<()> {
        // let ubo_layout_binding = [

        // ];
        // let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        //     .bindings(&ubo_layout_binding);

        // self.m_base.m_descriptor_infos.push(Descriptor { 
        //     layout: rhi.create_descriptor_set_layout(&layout_info)?,
        //     descriptor_set: vk::DescriptorSet::null(),
        // });
        Ok(())
    }

    fn setup_pipelines(&mut self ,rhi: &VulkanRHI)-> Result<()> {
        let vert_shader_module = rhi.create_shader_module(&MESH_VERT)?;
        let frag_shader_module = rhi.create_shader_module(&MESH_GBUFFER_FRAG)?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding_descriptions = &MeshVertex::get_binding_descriptions();
        let attribute_descriptions = &MeshVertex::get_attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let swapchain_info = rhi.get_swapchain_info();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(swapchain_info.viewport))
            .scissors(std::slice::from_ref(swapchain_info.scissor));

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

        let set_layouts = &self.m_base.get_descriptor_set_layouts();
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
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(self.m_base.m_framebuffer.render_pass)
            .subpass(0)
            .build();

        let pipeline = rhi.create_graphics_pipelines(vk::PipelineCache::null(), &[info])?[0];

        rhi.destroy_shader_module(vert_shader_module);
        rhi.destroy_shader_module(frag_shader_module);

        self.m_base.m_render_pipeline.push(RenderPipelineBase{
            layout: pipeline_layout,
            pipeline,
        });

        Ok(())
    }

    fn draw_object(&self, current_swapchain_image_index: usize) -> Result<()> {
        let rhi = self.m_base.m_base.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        let command_buffer = rhi.get_current_command_buffer();

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(swapchain_info.extent);

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue{ float32: [0.0, 0.0, 0.0, 1.0] },
            },
            vk::ClearValue{ 
                depth_stencil: vk::ClearDepthStencilValue{depth: 1.0, stencil: 0 },
            },
        ];
        
        let pipeline = &self.m_base.m_render_pipeline[0];
        
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.m_base.m_framebuffer.render_pass)
            .framebuffer(self.m_swapchain_framebuffers[current_swapchain_image_index])
            .render_area(render_area)
            .clear_values(&clear_values);

        rhi.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
        rhi.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        rhi.cmd_end_render_pass(command_buffer);
        Ok(())
    }
}


