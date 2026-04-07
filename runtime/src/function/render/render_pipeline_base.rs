use std::{cell::RefCell, rc::{Rc, Weak}};

use imgui_winit_support::WinitPlatform;

use crate::{function::{render::{debugdraw::debug_draw_manager::DebugDrawManager, interface::vulkan::vulkan_rhi::VulkanRHI, passes::{color_grading_pass::ColorGradingPass, combine_ui_pass::CombineUIPass, directional_light_pass::DirectionalLightShadowPass, fxaa_pass::FXAAPass, main_camera_pass::MainCameraPass, pick_pass::PickPass, point_light_pass::PointLightShadowPass, tone_mapping_pass::ToneMappingPass, ui_pass::UIPass}, render_resource::RenderResource}, ui::window_ui::WindowUI}, resource::config_manager::ConfigManager};

pub struct RenderPipelineCreateInfo<'a>{
    pub rhi : &'a Rc<RefCell<VulkanRHI>>,
    pub render_resource : &'a Rc<RefCell<RenderResource>>,
    pub enable_fxaa : bool,
    pub imgui_context : &'a Rc<RefCell<imgui::Context>>,
    pub imgui_platform : &'a Rc<RefCell<WinitPlatform>>,
    pub config_manager : &'a ConfigManager,
}

pub struct RenderPipelineBase{
    pub m_rhi : Weak<RefCell<VulkanRHI>>,

    pub m_directional_light_pass: DirectionalLightShadowPass,
    pub m_point_light_pass: PointLightShadowPass,
    pub m_main_camera_pass: MainCameraPass,
    pub m_tone_mapping_pass: ToneMappingPass,
    pub m_color_grading_pass: ColorGradingPass,
    pub m_fxaa_pass: FXAAPass,
    pub m_ui_pass: UIPass,
    pub m_combine_ui_pass: CombineUIPass,
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
        self.m_ui_pass.initialize_ui_render_backend(window_ui);
    }

    pub fn destroy(&self) {
        self.m_main_camera_pass.destroy();
    }
}