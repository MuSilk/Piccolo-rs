use std::{any::TypeId, cell::RefCell, collections::HashMap};

use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;

#[derive(Default, Clone, Copy)]
pub struct FrameBufferAttachment {
    pub image: vk::Image,
    pub mem: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format,
}

#[derive(Default)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub framebuffer: vk::Framebuffer,
    pub render_pass: vk::RenderPass,
    pub attachments: Vec<FrameBufferAttachment>,
}

#[derive(Default)]
pub struct Descriptor {
    pub layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
}

pub trait DescriptorLayout {
    fn new(rhi: &VulkanRHI) -> Result<vk::DescriptorSetLayout>;
}

#[derive(Default)]
pub struct DescriptorLayoutRegistry {
    m_descriptor_set_layouts: RefCell<HashMap<TypeId, vk::DescriptorSetLayout>>,
}

impl DescriptorLayoutRegistry {
    pub fn acquire<T: DescriptorLayout + 'static>(
        &self,
        rhi: &VulkanRHI,
    ) -> Result<vk::DescriptorSetLayout> {
        let type_id = TypeId::of::<T>();
        if let Some(layout) = self
            .m_descriptor_set_layouts
            .borrow()
            .get(&type_id)
            .copied()
        {
            return Ok(layout);
        }

        let layout = T::new(rhi)?;
        self.m_descriptor_set_layouts
            .borrow_mut()
            .insert(type_id, layout);
        Ok(layout)
    }
}

#[derive(Default)]
pub struct RenderPipelineBase {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

#[derive(Default)]
pub struct RenderPass {
    pub m_descriptor_infos: Vec<Descriptor>,
    pub m_render_pipeline: Vec<RenderPipelineBase>,
    pub m_framebuffer: Framebuffer,
}

impl RenderPass {
    pub fn create() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn get_render_pass(&self) -> &vk::RenderPass {
        &self.m_framebuffer.render_pass
    }
    pub fn get_framebuffer_image_views(&self) -> Vec<vk::ImageView> {
        self.m_framebuffer
            .attachments
            .iter()
            .map(|attachment| attachment.view)
            .collect::<Vec<_>>()
    }
}
