use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{function::{render::{debugdraw::debug_draw_manager::DebugDrawManager, interface::vulkan::vulkan_rhi::VulkanRHI, passes::{directional_light_pass::DirectionalLightShadowPass, main_camera_pass::MainCameraPass, pick_pass::PickPass, point_light_pass::PointLightShadowPass}, render_resource::RenderResource}, ui::window_ui::WindowUI}, resource::config_manager::ConfigManager};

pub struct RenderPipelineCreateInfo<'a>{
    pub rhi : &'a Rc<RefCell<VulkanRHI>>,
    pub render_resource : &'a Rc<RefCell<RenderResource>>,
    pub enable_fxaa : bool,
    pub config_manager : &'a ConfigManager,
}

pub struct RenderPipelineBase{
    pub m_rhi : Weak<RefCell<VulkanRHI>>,

    pub m_directional_light_pass: DirectionalLightShadowPass,
    pub m_point_light_pass: PointLightShadowPass,
    pub m_main_camera_pass: MainCameraPass,
    pub m_pick_pass: PickPass,
}

impl RenderPipelineBase{
    pub fn prepare_pass_data(
        &mut self, 
        debugdraw_manager: &mut DebugDrawManager,
        render_resource : &RenderResource,
    ){
        self.m_directional_light_pass.prepare_pass_data(render_resource);
        self.m_point_light_pass.prepare_pass_data(render_resource);
        self.m_main_camera_pass.prepare_pass_data(render_resource);
        self.m_pick_pass.prepare_pass_data(render_resource);
        debugdraw_manager.prepare_pass_data(render_resource);
    }   

    pub fn initialize_ui_render_backend(&mut self, window_ui: &Rc<RefCell<dyn WindowUI>>) {
        self.m_main_camera_pass.m_ui_pass.initialize_ui_render_backend(window_ui);
    }

    pub fn destroy(&self) {
        self.m_main_camera_pass.destroy();
    }
}