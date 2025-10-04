use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{interface::rhi::RHI, render_resource::{GlobalRenderResource, RenderResource}, render_resource_base::RenderResourceBase}};


pub struct RenderPipelineInitInfo<'a>{
    pub rhi: &'a Rc<RefCell<Box<dyn RHI>>>,
    pub enable_fxaa: bool,
    pub render_resource: &'a RenderResource,
}

#[derive(Default)]
pub struct RenderPipelineBase{
    pub m_rhi : Weak<RefCell<Box<dyn RHI>>>,
}

impl RenderPipelineBase{
    pub fn prepare_pass_data(&self, render_resource : &RenderResource){
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().prepare_pass_data(render_resource);
    }   
}