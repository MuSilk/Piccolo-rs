use std::{cell::RefCell, rc::{Rc, Weak}};

use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::{runtime::function::render::{debugdraw::debug_draw_primitive::DebugDrawVertex, interface::vulkan::vulkan_rhi::VulkanRHI}, shader::generated::shader::{DEBUGDRAW_FRAG, DEBUGDRAW_VERT}};

struct DebugDrawFramebufferAttachment {
    image: vk::Image,
    mem: vk::DeviceMemory,
    view: vk::ImageView,
    format: vk::Format,
}

#[derive(Default)]
pub struct DebugDrawFramebuffer{
    width: u32,
    height: u32,
    pub render_pass: vk::RenderPass,

    pub framebuffers: Vec<vk::Framebuffer>,
    attachments: Vec<DebugDrawFramebufferAttachment>,
}

pub struct DebugDrawPipelineBase {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub enum DebugDrawPipelineType {
    #[default]
    Point,
    Line,
    Triangle,
    PointNoDepthTest,
    LineNoDepthTest,
    TriangleNoDepthTest,
    EnumCount,
}

impl DebugDrawPipelineType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(DebugDrawPipelineType::Point),
            1 => Some(DebugDrawPipelineType::Line),
            2 => Some(DebugDrawPipelineType::Triangle),
            3 => Some(DebugDrawPipelineType::PointNoDepthTest),
            4 => Some(DebugDrawPipelineType::LineNoDepthTest),
            5 => Some(DebugDrawPipelineType::TriangleNoDepthTest),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct DebugDrawPipeline {
    m_pipeline_type: DebugDrawPipelineType,
    m_render_pipelines: Vec<DebugDrawPipelineBase>,
    m_framebuffer: DebugDrawFramebuffer,
    m_rhi: Weak<RefCell<VulkanRHI>>,
}

impl DebugDrawPipeline {
    pub fn create(pipeline_type: DebugDrawPipelineType, rhi: &Rc<RefCell<VulkanRHI>>, descriptor_set_layout: vk::DescriptorSetLayout) -> Result<Self> {
        let m_rhi = Rc::downgrade(rhi);
        let rhi = rhi.borrow();
        let swapchain_info = rhi.get_swapchain_info();
        setup_attachments();
        let render_pass = setup_render_pass(&rhi)?;
        let framebuffers = setup_framebuffer(&rhi, render_pass)?;
        let pipeline = setup_pipelines(&rhi, render_pass, descriptor_set_layout, pipeline_type)?;
        Ok(Self{
            m_pipeline_type : pipeline_type,
            m_render_pipelines: vec![pipeline],
            m_framebuffer: DebugDrawFramebuffer{
                width: swapchain_info.extent.width,
                height: swapchain_info.extent.height,
                render_pass,
                framebuffers,
                attachments: vec![],
            },
            m_rhi,
        })
    }

    pub fn recreate_after_swapchain(&mut self, rhi: &VulkanRHI) -> Result<()>{
        for framebuffer in self.m_framebuffer.framebuffers.drain(..){
            rhi.destroy_framebuffer(framebuffer);
        }
        self.m_framebuffer.framebuffers = setup_framebuffer(&rhi, self.m_framebuffer.render_pass)?;
        Ok(())
    }

    pub fn destroy(&self) {
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.borrow();
        self.m_framebuffer.framebuffers.iter().for_each(|f| rhi.destroy_framebuffer(*f));
        rhi.destroy_pipeline(self.m_render_pipelines[0].pipeline);
        rhi.destroy_pipeline_layout(self.m_render_pipelines[0].layout);
        rhi.destroy_render_pass(self.m_framebuffer.render_pass);
    }

    pub fn get_pipeline(&self) -> &DebugDrawPipelineBase {
        &self.m_render_pipelines[0]
    }

    pub fn get_framebuffer(&self) -> &DebugDrawFramebuffer {
        &self.m_framebuffer
    }
}

fn setup_attachments() {

}

fn setup_render_pass(rhi: &VulkanRHI) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(rhi.get_swapchain_info().image_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::LOAD)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(rhi.get_depth_image_info().format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::LOAD)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
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

    Ok(rhi.create_render_pass(&info)?)
}

fn setup_framebuffer(rhi: &VulkanRHI, render_pass : vk::RenderPass) -> Result<Vec<vk::Framebuffer>> {
    let swapchain_info = rhi.get_swapchain_info();
    let depth_image_info = rhi.get_depth_image_info();
    let framebuffers =  swapchain_info.image_views
        .iter()
        .map(|i| {
            let attachments = &[*i, *depth_image_info.image_view];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_info.extent.width)
                .height(swapchain_info.extent.height)
                .layers(1);

            rhi.create_framebuffer(&create_info)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(framebuffers)
}

fn setup_pipelines(rhi: &VulkanRHI, render_pass : vk::RenderPass, set_layout: vk::DescriptorSetLayout, pipeline_type: DebugDrawPipelineType)-> Result<DebugDrawPipelineBase> {

    let vert_shader_module = rhi.create_shader_module(&DEBUGDRAW_VERT)?;
    let frag_shader_module = rhi.create_shader_module(&DEBUGDRAW_FRAG)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[DebugDrawVertex::get_binding_descriptions()];
    let attribute_descriptions = DebugDrawVertex::get_attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let topology = match pipeline_type {
        DebugDrawPipelineType::Point | DebugDrawPipelineType::PointNoDepthTest => {
            vk::PrimitiveTopology::POINT_LIST
        },
        DebugDrawPipelineType::Line | DebugDrawPipelineType::LineNoDepthTest => {
            vk::PrimitiveTopology::LINE_LIST
        },
        _ => {
            vk::PrimitiveTopology::TRIANGLE_LIST
        }
    };

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(topology)
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

    let depth_test_enable = match pipeline_type {
        DebugDrawPipelineType::PointNoDepthTest | DebugDrawPipelineType::LineNoDepthTest | DebugDrawPipelineType::TriangleNoDepthTest => false,
        _ => true,
    };

    let depth_stencil_attachment = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(depth_test_enable)
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

    let set_layouts = &[set_layout];
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
        .depth_stencil_state(&depth_stencil_attachment)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

    let pipeline = rhi.create_graphics_pipelines(vk::PipelineCache::null(), &[info])?[0];

    rhi.destroy_shader_module(vert_shader_module);
    rhi.destroy_shader_module(frag_shader_module);

    Ok(DebugDrawPipelineBase { layout: pipeline_layout, pipeline })
}
