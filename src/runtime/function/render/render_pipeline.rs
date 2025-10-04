use std::rc::Rc;

use anyhow::Result;

use crate::runtime::function::{global::{global_context::RuntimeGlobalContext}, render::{interface::vulkan::vulkan_rhi::VulkanRHI, render_pipeline_base::{RenderPipelineBase, RenderPipelineCreateInfo}}};


pub struct RenderPipeline {
    pub m_base : RenderPipelineBase
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        Ok(RenderPipeline { 
            m_base: RenderPipelineBase {
                m_rhi: Rc::downgrade(create_info.rhi)
            }
        })
    }

    pub fn forward_render(&self) -> Result<()> {
        let rhi = self.m_base.m_rhi.upgrade().unwrap();
        {
            let mut rhi = rhi.borrow_mut();
            rhi.wait_for_fence()?;
            rhi.reset_command_pool()?;
            if rhi.prepare_before_pass(
                &|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi)
            )? {
                return Ok(());
            }
        }
        {
            let rhi = rhi.borrow();
            let global = RuntimeGlobalContext::global().borrow();
            let debugdraw_manager = &mut global.m_debugdraw_manager.borrow_mut();
            debugdraw_manager.draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi))?;
        }
        Ok(())
    }
}

impl RenderPipeline {
    fn pass_update_after_recreate_swapchain(&self, rhi: &VulkanRHI) {
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().update_after_recreate_swap_chain(rhi);
    }
}