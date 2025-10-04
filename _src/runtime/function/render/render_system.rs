use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{interface::{rhi::{RHIInitInfo, RHI}, vulkan::vulkan_rhi::VulkanRHI}, render_camera::RenderCamera, render_pipeline::RenderPipeline, render_pipeline_base::RenderPipelineInitInfo, render_resource::RenderResource, render_type::RenderPipelineType, window_system::WindowSystem}};

pub struct RenderSystemInitInfo<'a>{
    pub window_system: &'a WindowSystem,
}

pub struct RenderSystem{
    m_rhi: Option<Rc<RefCell<Box<dyn RHI>>>>,
    m_render_pipeline_type: RenderPipelineType,
    m_render_camera: RenderCamera,
    m_render_resource: RenderResource,
    m_render_pipeline: RenderPipeline,
}

unsafe impl Send for RenderSystem {}
unsafe impl Sync for RenderSystem {}

impl Default for RenderSystem {
    fn default() -> Self {
        Self {
            m_rhi: None,
            m_render_pipeline_type: RenderPipelineType::ForwardPipeline,
            m_render_camera: RenderCamera::default(),
            m_render_resource: RenderResource::default(),
            m_render_pipeline: RenderPipeline::default(),
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
        self.m_rhi = Some(Rc::new(RefCell::new(Box::new(vulkan_rhi))));

        
        let pipeline_init_info = RenderPipelineInitInfo {
            rhi: self.m_rhi.as_ref().unwrap(),
            enable_fxaa: false,
            render_resource: &self.m_render_resource,
        };

        self.m_render_pipeline.initialize(&pipeline_init_info);

        Ok(())
    }
    pub fn tick(&mut self, delta_time: f32){
        self.process_swap_date();
        self.m_rhi.as_mut().unwrap().borrow_mut().prepare_context();
        self.m_render_pipeline.m_base.prepare_pass_data(&self.m_render_resource);
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().tick(delta_time);
        match self.m_render_pipeline_type {
            RenderPipelineType::ForwardPipeline => {
                self.m_render_pipeline.forward_render(&mut self.m_render_resource).unwrap();
            },
            RenderPipelineType::DeferredPipeline => {
                self.m_render_pipeline.defferred_render(self.m_rhi.as_mut().unwrap().borrow_mut().as_mut(), &mut self.m_render_resource).unwrap();
            },
            _ => {panic!("Unknown render pipeline type")}
        }
    }
    pub fn clear(&mut self){
        if let Some(rhi) = &self.m_rhi {
            let mut rhi_borrow = rhi.borrow_mut();
            let vulkan_rhi = rhi_borrow.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
            vulkan_rhi.clear();
        }   
    }

    pub fn update_engine_content_viewport(&mut self, offset_x: f32, offset_y: f32, width: f32, height: f32){
        let mut rhi = self.m_rhi.as_ref().unwrap().borrow_mut();
        let vulkan_rhi = rhi.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
        vulkan_rhi.m_viewport.x = offset_x;
        vulkan_rhi.m_viewport.y = offset_y;
        vulkan_rhi.m_viewport.width = width;
        vulkan_rhi.m_viewport.height = height;
        vulkan_rhi.m_viewport.min_depth = 0.0;
        vulkan_rhi.m_viewport.max_depth = 1.0;

        self.m_render_camera.set_aspect(width/height);
    }

    pub fn get_rhi(&self) -> Rc<RefCell<Box<dyn RHI>>> {
        self.m_rhi.clone().unwrap()
    }
}

impl RenderSystem {
    fn process_swap_date(&self) {

    }
}