use std::{
    any::TypeId,
    cell::{LazyCell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::function::render::{
    interface::vulkan::vulkan_rhi::VulkanRHI, render_common::RenderMeshNode,
    render_resource::GlobalRenderResource,
};

#[derive(Default)]
pub struct VisiableNodes {
    pub p_directional_light_visible_mesh_nodes: Weak<RefCell<Vec<RenderMeshNode>>>,
    pub p_point_light_visible_mesh_nodes: Weak<RefCell<Vec<RenderMeshNode>>>,
    pub p_main_camera_visible_mesh_nodes: Weak<RefCell<Vec<RenderMeshNode>>>,
    // p_axis_node: RenderAxisNode,
}

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
pub struct DescriptorLayoutManager {
    m_descriptor_set_layouts: RefCell<HashMap<TypeId, vk::DescriptorSetLayout>>,
}

impl DescriptorLayoutManager {
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

pub static mut M_VISIABLE_NODES: LazyCell<RefCell<VisiableNodes>> =
    LazyCell::new(|| RefCell::new(VisiableNodes::default()));

#[derive(Default)]
pub struct RenderPass {
    pub m_global_render_resource: Weak<RefCell<GlobalRenderResource>>,

    pub m_descriptor_infos: Vec<Descriptor>,
    pub m_render_pipeline: Vec<RenderPipelineBase>,
    pub m_framebuffer: Framebuffer,
}

impl RenderPass {
    pub fn initialize(&mut self, global_render_resource: &Rc<RefCell<GlobalRenderResource>>) {
        self.m_global_render_resource = Rc::downgrade(global_render_resource);
    }

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

    #[allow(static_mut_refs)]
    pub fn m_visible_nodes() -> &'static RefCell<VisiableNodes> {
        unsafe { &M_VISIABLE_NODES }
    }
}
