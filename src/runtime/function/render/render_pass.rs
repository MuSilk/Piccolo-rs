use std::{ rc::{Rc}};

use vulkanalia::{prelude::v1_0::*};

use crate::runtime::function::render::{ render_pass_base::{RenderPassBase, RenderPassCommonInfo, RenderPassCreateInfo}};

pub const _MAIN_CAMERA_PASS_GBUFFER_A: usize = 0;
pub const _MAIN_CAMERA_PASS_GBUFFER_B: usize = 1;
pub const _MAIN_CAMERA_PASS_GBUFFER_C: usize = 2;
pub const _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD: usize = 3;
pub const _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN: usize = 4;
pub const _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD: usize = 5;
pub const _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN: usize = 6;
pub const _MAIN_CAMERA_PASS_DEPTH: usize = 7;
pub const _MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE: usize = 8;
pub const _MAIN_CAMERA_PASS_CUSTOM_ATTACHMENT_COUNT: usize = 5;
pub const _MAIN_CAMERA_PASS_POST_PROCESS_ATTACHMENT_COUNT: usize = 2;
pub const _MAIN_CAMERA_PASS_ATTACHMENT_COUNT:usize = 9;

pub const _MAIN_CAMERA_SUBPASS_BASEPASS: u32 = 0;
pub const _MAIN_CAMERA_SUBPASS_DEFERRED_LIGHTING: u32 = 1;
pub const _MAIN_CAMERA_SUBPASS_FORWARD_PROCESS: u32 = 2;
pub const _MAIN_CAMERA_SUBPASS_TONE_MAPPING: u32 = 3;
pub const _MAIN_CAMERA_SUBPASS_COLOR_GRADING: u32 = 4;
pub const _MAIN_CAMERA_SUBPASS_FXAA: u32 = 5;
pub const _MAIN_CAMERA_SUBPASS_UI: u32 = 6;
pub const _MAIN_CAMERA_SUBPASS_COMBINE_UI: u32 = 7;
pub const _MAIN_CAMERA_SUBPASS_COUNT: u32 = 8;

#[derive(Default,Clone, Copy)]
pub struct FrameBufferAttachment{
    pub image: vk::Image,
    pub mem: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format,
}

#[derive(Default)]
pub struct Framebuffer{
    pub width: u32,
    pub height: u32,
    pub framebuffer : vk::Framebuffer,
    pub render_pass: vk::RenderPass,
    pub attachments: Vec<FrameBufferAttachment>,
}

#[derive(Default)]
pub struct Descriptor{
    pub layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
}

pub struct RenderPipelineBase{
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

#[derive(Default)]
pub struct RenderPass{
    pub m_base : RenderPassBase,

    pub m_descriptor_infos: Vec<Descriptor>,
    pub m_render_pipeline: Vec<RenderPipelineBase>,
    pub m_framebuffer: Framebuffer,
}

impl RenderPass{
    pub fn set_common_info(&mut self, common_info: &RenderPassCommonInfo){
        self.m_base.m_rhi = Rc::downgrade(common_info.rhi);
    }

    pub fn create(info: &RenderPassCreateInfo) -> Self{
        Self {
            ..Default::default()
        }
    }
    pub fn get_render_pass(&self) -> &vk::RenderPass{
        &self.m_framebuffer.render_pass
    }
    pub fn get_framebuffer_image_views(&self) -> Vec<vk::ImageView>{
        self.m_framebuffer.attachments.iter()
            .map(|attachment| attachment.view).collect::<Vec<_>>()
    }
    pub fn get_descriptor_set_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
        self.m_descriptor_infos.iter()
            .map(|descriptor| descriptor.layout).collect::<Vec<_>>()
    }
} 