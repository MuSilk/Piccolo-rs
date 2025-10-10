

use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;

#[derive(Default)]
pub struct RenderPassCreateInfo{}

pub struct RenderPassCommonInfo<'a>{
    pub rhi: &'a Rc<RefCell<VulkanRHI>>,
}

#[derive(Default)]
pub struct RenderPassBase{
    pub m_rhi: Weak<RefCell<VulkanRHI>>,
}