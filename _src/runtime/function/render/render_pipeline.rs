use std::rc::Rc;

use anyhow::Result;

use crate::runtime::function::{global::global_context::RuntimeGlobalContext, render::{interface::{rhi::RHI, vulkan::vulkan_rhi::VulkanRHI}, render_pipeline_base::{RenderPipelineBase, RenderPipelineInitInfo}, render_resource::{RenderResource}}};

#[derive(Default)]
pub struct RenderPipeline {
    pub m_base : RenderPipelineBase
} 

impl RenderPipeline {
    pub fn initialize(&mut self, init_info: &RenderPipelineInitInfo) {
        self.m_base.m_rhi = Rc::downgrade(init_info.rhi);
    }

    pub fn forward_render(&self, render_resource: &mut RenderResource) -> Result<()> {
        let rhi = self.m_base.m_rhi.upgrade().unwrap();
        {
            let mut rhi = rhi.borrow_mut();
            let vulkan_rhi = rhi.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
            
            render_resource.reset_ring_buffer_offset(vulkan_rhi.get_current_frame_index());

            vulkan_rhi.wait_for_fence()?;
            vulkan_rhi.reset_command_pool()?;
            let is_recreate_swapchain = vulkan_rhi.prepare_before_pass(&||self.pass_update_after_recreate_swapchain())?;

            if is_recreate_swapchain {
                return Ok(());
            }
        }
        {
            let rhi = rhi.borrow();
            let vulkan_rhi = rhi.as_any().downcast_ref::<VulkanRHI>().unwrap();
            RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().draw(vulkan_rhi.m_current_swapchain_image_index as usize)?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&||self.pass_update_after_recreate_swapchain())?;
        }
        Ok(())
    }

    pub fn defferred_render(&self, rhi : &mut dyn RHI, render_resource: &mut RenderResource) -> Result<()> {
        let vulkan_rhi = rhi.as_any_mut().downcast_mut::<VulkanRHI>().unwrap();
        
        render_resource.reset_ring_buffer_offset(vulkan_rhi.get_current_frame_index());

        vulkan_rhi.wait_for_fence()?;

        vulkan_rhi.reset_command_pool()?;


        let is_recreate_swapchain = vulkan_rhi.prepare_before_pass(&||self.pass_update_after_recreate_swapchain())?;

        if is_recreate_swapchain {
            return Ok(());
        }

        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().draw(vulkan_rhi.m_current_swapchain_image_index as usize)?;

        rhi.submit_rendering(&||self.pass_update_after_recreate_swapchain())?;

        Ok(())
    }

    fn pass_update_after_recreate_swapchain(&self){
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().update_after_recreate_swap_chain();
    }
}