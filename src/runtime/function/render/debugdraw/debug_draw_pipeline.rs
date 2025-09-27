use std::sync::{Arc, Mutex, Weak};

use anyhow::Result;

use crate::{runtime::function::{global::global_context::RuntimeGlobalContext, render::{debugdraw::debug_draw_primitive::DebugDrawVertex, interface::{rhi::RHI, rhi_struct::{RHIAttachmentDescription, RHIAttachmentReference, RHIDescriptorSetLayout, RHIDescriptorSetLayoutBinding, RHIDescriptorSetLayoutCreateInfo, RHIDeviceMemory, RHIFramebuffer, RHIFramebufferCreateInfo, RHIGraphicsPipelineCreateInfo, RHIImage, RHIImageView, RHIPipeline, RHIPipelineColorBlendAttachmentState, RHIPipelineColorBlendStateCreateInfo, RHIPipelineDepthStencilStateCreateInfo, RHIPipelineDynamicStateCreateInfo, RHIPipelineInputAssemblyStateCreateInfo, RHIPipelineLayout, RHIPipelineLayoutCreateInfo, RHIPipelineMultisampleStateCreateInfo, RHIPipelineRasterizationStateCreateInfo, RHIPipelineShaderStageCreateInfo, RHIPipelineVertexInputStateCreateInfo, RHIPipelineViewportStateCreateInfo, RHIRenderPass, RHIRenderPassCreateInfo, RHISubPassDependency, RHISubpassDescription}}, render_type::{RHIAccessFlags, RHIAttachmentDescriptionFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIBlendFactor, RHIBlendOp, RHIColorComponentFlags, RHICompareOp, RHICullModeFlags, RHIDependencyFlags, RHIDescriptorSetLayoutCreateFlags, RHIDescriptorType, RHIDynamicState, RHIFormat, RHIFramebufferCreateFlags, RHIFrontFace, RHIImageLayout, RHILogicOp, RHIPipelineBindPoint, RHIPipelineColorBlendStateCreateFlags, RHIPipelineCreateFlags, RHIPipelineDynamicStateCreateFlags, RHIPipelineInputAssemblyStateCreateFlags, RHIPipelineLayoutCreateFlags, RHIPipelineMultisampleStateCreateFlags, RHIPipelineRasterizationStateCreateFlags, RHIPipelineShaderStageCreateFlags, RHIPipelineStageFlags, RHIPipelineVertexInputStateCreateFlags, RHIPipelineViewportStateCreateFlags, RHIPolygonMode, RHIPrimitiveTopology, RHIRenderPassCreateFlags, RHISampleCountFlags, RHIShaderStageFlags, RHISubpassDescriptionFlags, RHI_SUBPASS_EXTERNAL}}}, shader::generated::shader::{DEBUG_DRAW_FRAG, DEBUG_DRAW_VERT}};

struct DebugDrawFramebufferAttachment {
    image: Box<dyn RHIImage>,
    mem: Box<dyn RHIDeviceMemory>,
    view: Box<dyn RHIImageView>,
    format: RHIFormat
}

pub struct DebugDrawFramebuffer{
    width: u32,
    height: u32,
    pub render_pass: Box<dyn RHIRenderPass>,

    pub framebuffers: Vec<Box<dyn RHIFramebuffer>>,
    attachments: Vec<DebugDrawFramebufferAttachment>,
}

pub struct DebugDrawPipelineBase {
    pub layout: Box<dyn RHIPipelineLayout>,
    pub pipeline: Box<dyn RHIPipeline>,
}

#[repr(C)]
#[derive(Default)]
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
    pub m_pipeline_type: DebugDrawPipelineType,
    m_descriptor_set_layout: Option<Box<dyn RHIDescriptorSetLayout>>,
    m_render_pipelines: Vec<DebugDrawPipelineBase>,
    m_framebuffer: Option<DebugDrawFramebuffer>,
    m_rhi: Weak<Mutex<Box<dyn RHI>>>,
}

impl DebugDrawPipeline {
    pub fn initialize(&mut self) -> Result<()> {
        self.m_rhi = Arc::downgrade(&RuntimeGlobalContext::global().m_render_system.lock().unwrap().get_rhi());
        self.setup_attachments();
        self.setup_render_pass()?;
        self.setup_framebuffer();
        self.setup_descriptor_layout()?;
        self.setup_pipelines()?;
        Ok(())
    }

    pub fn recreate_after_swapchain(&mut self){
        for framebuffer in self.m_framebuffer.as_mut().unwrap().framebuffers.drain(..){
            self.m_rhi.upgrade().unwrap().lock().unwrap().destroy_framebuffer(framebuffer);
        }
        self.setup_framebuffer();
    }
}

impl DebugDrawPipeline {
    fn setup_attachments(&self) {

    }

    fn setup_render_pass(&mut self) -> Result<()> {
        let color_attachment_descriptor = RHIAttachmentDescription {
            flags : RHIAttachmentDescriptionFlags::empty(),
            format: self.m_rhi.upgrade().unwrap().lock().unwrap().get_swap_chain_info().image_format,
            samples: RHISampleCountFlags::_1,
            load_op: RHIAttachmentLoadOp::LOAD,
            store_op: RHIAttachmentStoreOp::STORE,
            stencil_load_op: RHIAttachmentLoadOp::DONT_CARE,
            stencil_store_op: RHIAttachmentStoreOp::DONT_CARE,
            initial_layout: RHIImageLayout::PRESENT_SRC_KHR,
            final_layout: RHIImageLayout::PRESENT_SRC_KHR,
        };
        let color_attachment_reference = RHIAttachmentReference {
            attachment: 0,
            layout: RHIImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let depth_attachment_descriptor = RHIAttachmentDescription {
            flags : RHIAttachmentDescriptionFlags::empty(),
            format: self.m_rhi.upgrade().unwrap().lock().unwrap().get_depth_image_info().format,
            samples: RHISampleCountFlags::_1,
            load_op: RHIAttachmentLoadOp::LOAD,
            store_op: RHIAttachmentStoreOp::STORE,
            stencil_load_op: RHIAttachmentLoadOp::DONT_CARE,
            stencil_store_op: RHIAttachmentStoreOp::DONT_CARE,
            initial_layout: RHIImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            final_layout: RHIImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let depth_attachment_reference = RHIAttachmentReference {
            attachment: 1,
            layout: RHIImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = RHISubpassDescription {
            flags: RHISubpassDescriptionFlags::empty(),
            pipeline_bind_point: RHIPipelineBindPoint::GRAPHICS,
            color_attachments: &[color_attachment_reference],
            depth_stencil_attachment: &depth_attachment_reference,
            input_attachments: &[],
            preserve_attachments: &[],
            resolve_attachments: &[],
        };

        let attachments = [color_attachment_descriptor, depth_attachment_descriptor];
        let dependencies = [
            RHISubPassDependency {
                src_subpass: 0,
                dst_subpass: RHI_SUBPASS_EXTERNAL,
                src_stage_mask: RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | RHIPipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_stage_mask: RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | RHIPipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: RHIAccessFlags::COLOR_ATTACHMENT_WRITE | RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_access_mask: RHIAccessFlags::COLOR_ATTACHMENT_WRITE | RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dependency_flags: RHIDependencyFlags::empty(),
            }
        ];

        let render_pass_create_info = RHIRenderPassCreateInfo {
            flags: RHIRenderPassCreateFlags::empty(),
            attachments: &attachments,
            subpasses: &[subpass],
            dependencies: &dependencies,
        };
        self.m_framebuffer.as_mut().unwrap().render_pass = self.m_rhi.upgrade().unwrap().lock().unwrap().create_render_pass(&render_pass_create_info)?;
        Ok(())
    }

    fn setup_framebuffer(&mut self) {
        let rhi_guard = self.m_rhi.upgrade().unwrap();
        let rhi_guard = rhi_guard.lock().unwrap();
        let image_views = rhi_guard.get_swap_chain_info().image_views;
        self.m_framebuffer.as_mut().unwrap().framebuffers = image_views.iter().map(|image_view| {
            let attachments = [
                &image_view,
                rhi_guard.get_depth_image_info().image_view,
            ];
            let framebuffer_create_info = RHIFramebufferCreateInfo {
                flags: RHIFramebufferCreateFlags::empty(),
                render_pass: &self.m_framebuffer.as_ref().unwrap().render_pass,
                attachments: &attachments,
                width: rhi_guard.get_swap_chain_info().extent.width,
                height: rhi_guard.get_swap_chain_info().extent.height,
                layers: 1,
            };
            rhi_guard.create_framebuffer(&framebuffer_create_info).unwrap()
        }).collect::<Vec<_>>();
    }

    fn setup_descriptor_layout(&mut self) -> Result<()> {
        let ubo_layout_binding = [
            RHIDescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::VERTEX,
                p_immutable_samplers: None,
            },
            RHIDescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: RHIDescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::VERTEX,
                p_immutable_samplers: None,
            },
            RHIDescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: RHIDescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: RHIShaderStageFlags::FRAGMENT,
                p_immutable_samplers: None,
            },
        ];

        let layout_info = RHIDescriptorSetLayoutCreateInfo {
            flags: RHIDescriptorSetLayoutCreateFlags::empty(),
            bindings: &ubo_layout_binding,
        };

        self.m_descriptor_set_layout = Some(
            self.m_rhi.upgrade().unwrap().lock().unwrap().create_descriptor_set_layout(&layout_info)?
        );
        Ok(())
    }

    fn setup_pipelines(&mut self)-> Result<()> {
        let rhi = self.m_rhi.upgrade().unwrap();
        let rhi = rhi.lock().unwrap();
        let pipeline_layout_create_info = RHIPipelineLayoutCreateInfo {
            flags: RHIPipelineLayoutCreateFlags::empty(),
            set_layouts: &[self.m_descriptor_set_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        };
        let pipeline_layout = self.m_rhi.upgrade().unwrap().lock().unwrap().create_pipeline_layout(&pipeline_layout_create_info).unwrap();
        
        let vert_shader_module = self.m_rhi.upgrade().unwrap().lock().unwrap().create_shader_module(DEBUG_DRAW_VERT)?;
        let frag_shader_module = self.m_rhi.upgrade().unwrap().lock().unwrap().create_shader_module(DEBUG_DRAW_FRAG)?;
        
        let vert_pipeline_shader_stage_info = RHIPipelineShaderStageCreateInfo {
            flags : RHIPipelineShaderStageCreateFlags::empty(),
            stage: RHIShaderStageFlags::VERTEX,
            module: &vert_shader_module,
            name: "main",
            specialization_info: None,
        };
        let frag_pipeline_shader_stage_info = RHIPipelineShaderStageCreateInfo {
            flags : RHIPipelineShaderStageCreateFlags::empty(),
            stage: RHIShaderStageFlags::FRAGMENT,
            module: &frag_shader_module,
            name: "main",
            specialization_info: None,
        };
        let shader_stage = [
            vert_pipeline_shader_stage_info,
            frag_pipeline_shader_stage_info,
        ];

        let vertex_input_state_create_info = RHIPipelineVertexInputStateCreateInfo {
            flags: RHIPipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_descriptions: &DebugDrawVertex::get_binding_descriptions(),
            vertex_attribute_descriptions: &DebugDrawVertex::get_attribute_descriptions(),
        };

        let topology = match self.m_pipeline_type {
            DebugDrawPipelineType::Point | DebugDrawPipelineType::PointNoDepthTest => RHIPrimitiveTopology::POINT_LIST,
            DebugDrawPipelineType::Line | DebugDrawPipelineType::LineNoDepthTest => RHIPrimitiveTopology::LINE_LIST,
            DebugDrawPipelineType::Triangle | DebugDrawPipelineType::TriangleNoDepthTest => RHIPrimitiveTopology::TRIANGLE_LIST,
            _ => RHIPrimitiveTopology::LINE_LIST,
        };

        let input_assembly_create_info = RHIPipelineInputAssemblyStateCreateInfo {
            flags: RHIPipelineInputAssemblyStateCreateFlags::empty(),
            topology,
            primitive_restart_enable: false,
        };

        let viewport_state_create_info = RHIPipelineViewportStateCreateInfo {
            flags: RHIPipelineViewportStateCreateFlags::empty(),
            viewports: &[rhi.get_swap_chain_info().viewport],
            scissors: &[rhi.get_swap_chain_info().scissor],
        };

        let rasterization_state_create_info = RHIPipelineRasterizationStateCreateInfo {
            flags: RHIPipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: RHIPolygonMode::FILL,
            line_width: 1.0,
            cull_mode: RHICullModeFlags::NONE,
            front_face: RHIFrontFace::CLOCKWISE,
            depth_bias_enable: false,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
        };

        let multisample_state_create_info = RHIPipelineMultisampleStateCreateInfo {
            flags: RHIPipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: RHISampleCountFlags::_1,
            sample_shading_enable: false,
            min_sample_shading: 1.0,
            sample_mask: None,
            alpha_to_coverage_enable: false,
            alpha_to_one_enable: false,
        };

        let color_blend_attachment_state = RHIPipelineColorBlendAttachmentState {
            blend_enable: true,
            src_color_blend_factor: RHIBlendFactor::SRC_ALPHA,
            dst_color_blend_factor: RHIBlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: RHIBlendOp::ADD,
            src_alpha_blend_factor: RHIBlendFactor::ONE,
            dst_alpha_blend_factor: RHIBlendFactor::ZERO,
            alpha_blend_op: RHIBlendOp::ADD,
            color_write_mask: RHIColorComponentFlags::R
                | RHIColorComponentFlags::G
                | RHIColorComponentFlags::B
                | RHIColorComponentFlags::A,
        };

        let color_blend_state_create_info = RHIPipelineColorBlendStateCreateInfo {
            flags: RHIPipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: false,
            logic_op: RHILogicOp::COPY,
            attachments: &[&color_blend_attachment_state],
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };

        let depth_test_enable = match self.m_pipeline_type {
            DebugDrawPipelineType::PointNoDepthTest | DebugDrawPipelineType::LineNoDepthTest | DebugDrawPipelineType::TriangleNoDepthTest => false,
            _ => true,
        };

        let depth_stencil_create_info = RHIPipelineDepthStencilStateCreateInfo {
            flags: 0,
            depth_test_enable: depth_test_enable,
            depth_write_enable: true,
            depth_compare_op: RHICompareOp::LESS,
            depth_bounds_test_enable: false,
            stencil_test_enable: false,
            front: Default::default(),
            back: Default::default(),
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
        };

        let dynamic_states = [RHIDynamicState::VIEWPORT, RHIDynamicState::SCISSOR];
        let pipeline_dynamic_state_create_info = RHIPipelineDynamicStateCreateInfo {
            flags: RHIPipelineDynamicStateCreateFlags::empty(),
            dynamic_states: &dynamic_states,
        };

        let pipeline_create_info = RHIGraphicsPipelineCreateInfo {
            flags: RHIPipelineCreateFlags::empty(),
            stages: &shader_stage,
            vertex_input_state: &vertex_input_state_create_info,
            input_assembly_state: &input_assembly_create_info,
            tessellation_state: None,
            viewport_state: &viewport_state_create_info,
            rasterization_state: &rasterization_state_create_info,
            multisample_state: &multisample_state_create_info,
            depth_stencil_state: Some(&depth_stencil_create_info),
            color_blend_state: &color_blend_state_create_info,
            dynamic_state: Some(&pipeline_dynamic_state_create_info),
            layout: &pipeline_layout,
            render_pass: &self.m_framebuffer.as_ref().unwrap().render_pass,
            subpass: 0,
            base_pipeline_handle: None,
            base_pipeline_index: -1,
        };
        
        let pipelines = rhi.create_graphics_pipelines(&[pipeline_create_info])?.remove(0);
        self.m_render_pipelines.push(DebugDrawPipelineBase {
            layout: pipeline_layout,
            pipeline: pipelines,
        });

        rhi.destroy_shader_module(vert_shader_module);
        rhi.destroy_shader_module(frag_shader_module);

        Ok(())
    }

    pub fn destroy(&self) {

    }

    pub fn get_pipeline(&self) -> &DebugDrawPipelineBase {
        &self.m_render_pipelines[0]
    }

    pub fn get_framebuffer(&self) -> &DebugDrawFramebuffer {
        &self.m_framebuffer.as_ref().unwrap()
    }
}