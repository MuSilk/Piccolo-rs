use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::function::{global::global_context::RuntimeGlobalContext, render::{interface::vulkan::vulkan_rhi::VulkanRHI, passes::main_camera_pass::MainCameraPass, render_resource::RenderResource}};

pub struct RenderPipelineCreateInfo<'a>{
    pub rhi : &'a Rc<RefCell<VulkanRHI>>
}

pub struct RenderPipelineBase{
    pub m_rhi : Weak<RefCell<VulkanRHI>>,

    pub m_main_camera_pass: MainCameraPass,
}

impl RenderPipelineBase{
    pub fn prepare_pass_data(&self, render_resource : &RenderResource){
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().prepare_pass_data(render_resource);
    }   

    pub fn destroy(&self) {
        self.m_main_camera_pass.destroy();
    }
}