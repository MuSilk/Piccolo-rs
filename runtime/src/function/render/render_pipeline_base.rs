use std::{cell::RefCell, rc::{Rc, Weak}};

use imgui_winit_support::WinitPlatform;

use crate::function::{global::global_context::RuntimeGlobalContext, render::{interface::vulkan::vulkan_rhi::VulkanRHI, passes::{color_grading_pass::ColorGradingPass, combine_ui_pass::CombineUIPass, fxaa_pass::FXAAPass, main_camera_pass::MainCameraPass, ui_pass::UIPass}, render_resource::RenderResource}};

pub struct RenderPipelineCreateInfo<'a>{
    pub rhi : &'a Rc<RefCell<VulkanRHI>>,
    pub render_resource : &'a Rc<RefCell<RenderResource>>,
    pub enable_fxaa : bool,
    pub imgui_context : &'a Rc<RefCell<imgui::Context>>,
    pub imgui_platform : &'a Rc<RefCell<WinitPlatform>>,
}

pub struct RenderPipelineBase{
    pub m_rhi : Weak<RefCell<VulkanRHI>>,

    pub m_main_camera_pass: MainCameraPass,
    pub m_color_grading_pass: ColorGradingPass,
    pub m_fxaa_pass: FXAAPass,
    pub m_ui_pass: UIPass,
    pub m_combine_ui_pass: CombineUIPass,
}

impl RenderPipelineBase{
    pub fn prepare_pass_data(&mut self, render_resource : &RenderResource){
        self.m_main_camera_pass.prepare_pass_data(render_resource);
        RuntimeGlobalContext::get_debugdraw_manager().borrow_mut().prepare_pass_data(render_resource);
    }   

    pub fn destroy(&self) {
        self.m_main_camera_pass.destroy();
    }
}