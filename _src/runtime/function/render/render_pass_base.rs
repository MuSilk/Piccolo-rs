

use std::{cell::RefCell, rc::Weak};

use crate::runtime::function::{render::{interface::rhi::RHI, render_resource::RenderResource, render_resource_base::RenderResourceBase}, ui::window_ui::WindowUI};

pub struct RenderPassInitInfo{}

pub struct RenderPassCommonInfo{
    pub rhi: Weak<RefCell<Box<dyn RHI>>>,
    pub render_resource: Weak<RefCell<RenderResource>>
}

pub trait RenderPassBase{
    fn initialize(&mut self, init_info: RenderPassInitInfo);
    fn post_initialize(&self);
    fn set_common_info(&mut self, common_info: RenderPassCommonInfo);
    fn prepare_pass_data(render_resource: &RenderResourceBase){}
    fn initial_ui_render_backend(window_ui: &dyn WindowUI){}
}