

use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::function::render::{interface::vulkan::vulkan_rhi::VulkanRHI, render_resource::RenderResource};

#[derive(Default)]
pub struct RenderPassCreateInfo{}

pub struct RenderPassCommonInfo<'a>{
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
    pub render_resource: &'a Rc<RefCell<RenderResource>>,
}

#[derive(Default)]
pub struct RenderPassBase{
    pub m_rhi: Weak<RefCell<VulkanRHI>>,
    pub m_render_resource: Weak<RefCell<RenderResource>>,
}