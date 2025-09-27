use std::sync::{Arc, Weak};

use crate::runtime::function::render::{self, interface::{rhi::RHI, rhi_struct::{RHIDescriptorSet, RHIDescriptorSetLayout, RHIDeviceMemory, RHIFramebuffer, RHIImage, RHIImageView, RHIPipeline, RHIPipelineLayout, RHIRenderPass}}, render_pass_base::{RenderPassBase, RenderPassCommonInfo, RenderPassInitInfo}, render_resource::{GlobalRenderResource, RenderResource}, render_resource_base::RenderResourceBase, render_type::RHIFormat};

struct FrameBufferAttachment{
    image: Box<dyn RHIImage>,
    mem: Box<dyn RHIDeviceMemory>,
    view: Box<dyn RHIImageView>,
    format: RHIFormat,
}

struct Framebuffer{
    width: i32,
    height: i32,
    framebuffer : Box<dyn RHIFramebuffer>,
    render_pass: Box<dyn RHIRenderPass>,
    attachments: Vec<FrameBufferAttachment>,
}

struct Descriptor{
    layout: Box<dyn RHIDescriptorSetLayout>,
    descriptor_set: Box<dyn RHIDescriptorSet>,
}

struct RenderPipelineBase{
    layout: Box<dyn RHIPipelineLayout>,
    pipeline: Box<dyn RHIPipeline>,
}

pub struct RenderPass{
    m_rhi: Weak<Box<dyn RHI>>,
    m_render_resource: Weak<RenderResource>,

    m_global_render_resource: Weak<GlobalRenderResource>,

    m_descriptor_infos: Vec<Descriptor>,
    m_render_pipeline: Vec<RenderPipelineBase>,
    m_framebuffer: Framebuffer,
}

pub trait RenderPassTrait : RenderPassBase{
    fn get_ref(&self) -> &RenderPass;
    fn get_mut(&mut self) -> &mut RenderPass;
    fn set_common_info(&mut self, common_info: RenderPassCommonInfo){
        let render_pass = self.get_mut();
        render_pass.m_rhi = common_info.rhi;
        render_pass.m_render_resource = common_info.render_resource;
    }
    fn initialize(&mut self, init_info: RenderPassInitInfo){
        let render_pass = self.get_mut();
        let render_resource = render_pass.m_render_resource.upgrade().unwrap();
        render_pass.m_global_render_resource = Arc::downgrade(&render_resource.m_global_render_resource);
    }
    fn post_initialize(&self){}
    fn get_render_pass(&self) -> &dyn RHIRenderPass{
        self.get_ref().m_framebuffer.render_pass.as_ref()
    }
    fn get_framebuffer_image_views(&self) -> Vec<&Box<dyn RHIImageView>>{
        self.get_ref().m_framebuffer.attachments.iter()
            .map(|attachment| &attachment.view).collect::<Vec<_>>()
    }
    fn get_descriptor_set_layouts(&self) -> Vec<&Box<dyn RHIDescriptorSetLayout>> {
        self.get_ref().m_descriptor_infos.iter()
            .map(|descriptor| &descriptor.layout).collect::<Vec<_>>()
    }
} 