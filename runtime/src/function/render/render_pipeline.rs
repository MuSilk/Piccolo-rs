use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::function::{global::global_context::RuntimeGlobalContext, render::{interface::vulkan::vulkan_rhi::VulkanRHI, passes::{combine_ui_pass::{CombineUIPass, CombineUIPassInitInfo}, main_camera_pass::{MainCameraPass, MainCameraPassInitInfo}}, render_pass::_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD, render_pass_base::RenderPassCommonInfo, render_pipeline_base::{RenderPipelineBase, RenderPipelineCreateInfo}, render_resource::RenderResource}};


pub struct RenderPipeline {
    pub m_base : RefCell<RenderPipelineBase>
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        let mut m_main_camera_pass = MainCameraPass::default();
        let mut m_combine_ui_pass = CombineUIPass::default();

        let render_pass_common_info = RenderPassCommonInfo {
            rhi: create_info.rhi,
            render_resource: create_info.render_resource,
        };
        m_main_camera_pass.m_render_pass.set_common_info(&render_pass_common_info);
        m_combine_ui_pass.m_render_pass.set_common_info(&render_pass_common_info);

        m_main_camera_pass.initialize(&MainCameraPassInitInfo {
            rhi: create_info.rhi
        })?;

        m_combine_ui_pass.initialize(&CombineUIPassInitInfo {
            rhi: create_info.rhi,
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            scene_input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD]
        })?;

        Ok(RenderPipeline {
            m_base: RefCell::new(RenderPipelineBase {
                m_rhi: Rc::downgrade(create_info.rhi),
                m_main_camera_pass,
                m_combine_ui_pass,
            })
        })
    }

    pub fn forward_render(&mut self, render_resource: &mut RenderResource) -> Result<()> {
        let rhi = self.m_base.borrow().m_rhi.upgrade().unwrap();
        render_resource.reset_ring_buffer_offset(rhi.borrow().get_current_frame_index());
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
            self.m_base.borrow().m_main_camera_pass.draw(
                &self.m_base.borrow().m_combine_ui_pass,
                rhi.get_current_swapchain_image_index()
            )?;

            let debugdraw_manager = &mut global.m_debugdraw_manager.borrow_mut();
            debugdraw_manager.draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi))?;
        }
        Ok(())
    }

    pub fn defferred_render(&self, render_resource: &mut RenderResource) -> Result<()> {
        let rhi = self.m_base.borrow().m_rhi.upgrade().unwrap();
        render_resource.reset_ring_buffer_offset(rhi.borrow().get_current_frame_index());
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
        let image_views = self.m_base.borrow().m_main_camera_pass.m_render_pass.get_framebuffer_image_views();
        self.m_base.borrow_mut().m_combine_ui_pass.update_after_framebuffer_recreate(
            rhi,
            image_views[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD]
        ).unwrap();
        RuntimeGlobalContext::global().borrow().m_debugdraw_manager.borrow_mut().update_after_recreate_swap_chain(rhi);
    }
}