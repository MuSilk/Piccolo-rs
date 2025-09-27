use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use anyhow::Result;

use crate::runtime::function::render::{interface::{rhi::{RHIInitInfo, RHI}, vulkan::vulkan_rhi::VulkanRHI}, render_type::RenderPipelineType, window_system::WindowSystem};

pub struct RenderSystemInitInfo<'a>{
    pub window_system: &'a WindowSystem,
}

pub struct RenderSystem{
    rhi: Option<Arc<Mutex<Box<dyn RHI>>>>,
    m_render_pipeline_type: RenderPipelineType,
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self {
            rhi: None,
            m_render_pipeline_type: RenderPipelineType::FORWARD_PIPELINE,
        }
    }
}

impl RenderSystem {
    pub fn initialize(&mut self, init_info: RenderSystemInitInfo) -> Result<()> {
        let rhi_init_info = RHIInitInfo {
            window_system: init_info.window_system,
        };
        let mut vulkan_rhi = VulkanRHI::default();
        vulkan_rhi.initialize(rhi_init_info)?;
        self.rhi = Some(Arc::new(Mutex::new(Box::new(vulkan_rhi))));

        Ok(())
    }
    pub fn tick(delta_time: f32){
        unimplemented!();
    }
    pub fn clear(){
        unimplemented!();
    }

    pub fn get_rhi(&self) -> Arc<Mutex<Box<dyn RHI>>> {
        self.rhi.clone().unwrap()
    }
}