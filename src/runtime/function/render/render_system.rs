use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{interface::{rhi::RHICreateInfo, vulkan::vulkan_rhi::VulkanRHI}, render_pipeline::RenderPipeline, render_pipeline_base::RenderPipelineCreateInfo, render_type::RenderPipelineType, window_system::WindowSystem}};

pub struct RenderSystemCreateInfo<'a>{
    pub window_system: &'a WindowSystem,
}

pub struct RenderSystem{
    pub m_rhi: Rc<RefCell<VulkanRHI>>,
    m_render_pipeline_type: RenderPipelineType,
    // m_render_camera: RenderCamera,
    // m_render_resource: RenderResource,
    // m_render_pipeline: RenderPipeline,
    m_render_pipeline: RenderPipeline,
}

impl RenderSystem {
    pub fn create(create_info: RenderSystemCreateInfo) -> Result<Self> {
        let rhi_create_info = RHICreateInfo {
            window_system: create_info.window_system,
        };
        let vulakn_rhi = VulkanRHI::create(&rhi_create_info)?;
        let vulkan_rhi = Rc::new(RefCell::new(vulakn_rhi));

        let create_info = RenderPipelineCreateInfo {
            rhi : &vulkan_rhi
        };
        let render_pipeline = RenderPipeline::create(&create_info)?;

        Ok(Self {
            m_rhi: vulkan_rhi, 
            m_render_pipeline_type: RenderPipelineType::ForwardPipeline,
            m_render_pipeline: render_pipeline
        })
    }
    pub fn tick(&mut self, delta_time: f32) -> Result<()>{
        // self.process_swap_date();
        self.m_rhi.borrow_mut().prepare_context();
        // self.m_render_pipeline.m_base.prepare_pass_data(&self.m_render_resource);
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().tick(delta_time);
        // match self.m_render_pipeline_type {
        //     RenderPipelineType::ForwardPipeline => {
                self.m_render_pipeline.forward_render().unwrap();
        //     },
        //     RenderPipelineType::DeferredPipeline => {
        //         self.m_render_pipeline.defferred_render(self.m_rhi.as_mut().unwrap().borrow_mut().as_mut(), &mut self.m_render_resource).unwrap();
        //     },
        //     _ => {panic!("Unknown render pipeline type")}
        // }
        Ok(())
    }
    pub fn clear(&mut self){
        // if let Some(rhi) = &self.m_rhi {
        //     let mut rhi_borrow = rhi.borrow_mut();
        //     let vulkan_rhi = rhi_borrow.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
        //     vulkan_rhi.clear();
        // }   
    }

    pub fn update_engine_content_viewport(&mut self, offset_x: f32, offset_y: f32, width: f32, height: f32){
        let mut rhi = self.m_rhi.as_ref().borrow_mut();
        rhi.m_data.m_viewport.x = offset_x;
        rhi.m_data.m_viewport.y = offset_y;
        rhi.m_data.m_viewport.width = width;
        rhi.m_data.m_viewport.height = height;

        // self.m_render_camera.set_aspect(width/height);
    }

    pub fn get_rhi(&self) -> &Rc<RefCell<VulkanRHI>> {
        &self.m_rhi
    }
}

impl RenderSystem {
    fn process_swap_date(&self) {

    }
}