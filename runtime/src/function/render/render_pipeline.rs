use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::function::{global::global_context::RuntimeGlobalContext, render::{interface::vulkan::vulkan_rhi::VulkanRHI, passes::main_camera_pass::{MainCameraPass, MainCameraPassCreateInfo}, render_pipeline_base::{RenderPipelineBase, RenderPipelineCreateInfo}}};


pub struct RenderPipeline {
    pub m_base : RefCell<RenderPipelineBase>
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        Ok(RenderPipeline { 
            m_base: RefCell::new(RenderPipelineBase {
                m_rhi: Rc::downgrade(create_info.rhi),
                m_main_camera_pass: MainCameraPass::create(&MainCameraPassCreateInfo {rhi: create_info.rhi})?,
            })
        })
    }

    pub fn forward_render(&mut self) -> Result<()> {
        let rhi = self.m_base.borrow().m_rhi.upgrade().unwrap();
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
            self.m_base.borrow().m_main_camera_pass.draw(rhi.get_current_swapchain_image_index())?;

            let debugdraw_manager = &mut global.m_debugdraw_manager.borrow_mut();
            debugdraw_manager.draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi))?;
        }
        Ok(())
    }

    pub fn defferred_render(&self) -> Result<()> {
        let rhi = self.m_base.borrow().m_rhi.upgrade().unwrap();
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
        self.m_base.borrow_mut().m_main_camera_pass.recreate_after_swapchain(rhi).unwrap();
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().update_after_recreate_swap_chain(rhi);
    }
}