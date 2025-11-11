use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{core::math::vector2::Vector2, function::{global::global_context::RuntimeGlobalContext, render::{interface::vulkan::vulkan_rhi::VulkanRHI, passes::{color_grading_pass::{ColorGradingPass, ColorGradingPassInitInfo}, combine_ui_pass::{CombineUIPass, CombineUIPassInitInfo}, directional_light_pass::{DirectionalLightShadowPass, DirectionalLightShadowPassInitInfo}, fxaa_pass::{FXAAPass, FXAAPassInitInfo}, main_camera_pass::{LayoutType, MainCameraPass, MainCameraPassInitInfo}, pick_pass::{PickPass, PickPassInitInfo}, point_light_pass::{PointLightShadowPass, PointLightShadowPassInitInfo}, tone_mapping_pass::{ToneMappingInitInfo, ToneMappingPass}, ui_pass::{UIPass, UIPassInitInfo}}, render_pass::{_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD}, render_pass_base::RenderPassCommonInfo, render_pipeline_base::{RenderPipelineBase, RenderPipelineCreateInfo}, render_resource::RenderResource}}};


pub struct RenderPipeline {
    pub m_base : RefCell<RenderPipelineBase>
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        let mut m_directional_light_pass = DirectionalLightShadowPass::default();
        let mut m_point_light_pass = PointLightShadowPass::default();
        let mut m_main_camera_pass = MainCameraPass::default();
        let mut m_tone_mapping_pass = ToneMappingPass::default();
        let mut m_color_grading_pass = ColorGradingPass::default();
        let mut m_fxaa_pass = FXAAPass::default();
        let mut m_ui_pass = UIPass::default();
        let mut m_combine_ui_pass = CombineUIPass::default();
        let mut m_pick_pass = PickPass::default();

        let common_info = RenderPassCommonInfo {
            rhi: create_info.rhi,
            render_resource: create_info.render_resource,
        };
        m_directional_light_pass.m_render_pass.set_common_info(&common_info);
        m_point_light_pass.m_render_pass.set_common_info(&common_info);
        m_main_camera_pass.m_render_pass.set_common_info(&common_info);
        m_tone_mapping_pass.m_render_pass.set_common_info(&common_info);
        m_color_grading_pass.m_render_pass.set_common_info(&common_info);
        m_fxaa_pass.m_render_pass.set_common_info(&common_info);
        m_ui_pass.m_render_pass.set_common_info(&common_info);
        m_combine_ui_pass.m_render_pass.set_common_info(&common_info);
        m_pick_pass.m_render_pass.set_common_info(&common_info);

        m_directional_light_pass.initialize(&DirectionalLightShadowPassInitInfo {
            rhi: create_info.rhi,
        })?;

        m_point_light_pass.initialize(&PointLightShadowPassInitInfo {
            rhi: create_info.rhi,
        })?;

        m_main_camera_pass.m_directional_light_shadow_color_image_view = 
            m_directional_light_pass.m_render_pass.m_framebuffer.attachments[0].view;

        m_main_camera_pass.m_point_light_shadow_color_image_view = 
            m_point_light_pass.m_render_pass.m_framebuffer.attachments[0].view;

        m_main_camera_pass.initialize(&MainCameraPassInitInfo {
            rhi: create_info.rhi,
            enable_fxaa: create_info.enable_fxaa,
        })?;

        let descriptor_layouts = m_main_camera_pass.m_render_pass.get_descriptor_set_layouts();
        m_directional_light_pass.set_per_mesh_layout(descriptor_layouts[LayoutType::PerMesh as usize]);
        m_point_light_pass.set_per_mesh_layout(descriptor_layouts[LayoutType::PerMesh as usize]);

        m_directional_light_pass.post_initialize(&DirectionalLightShadowPassInitInfo {
            rhi: create_info.rhi,
        })?;
        m_point_light_pass.post_initialize(&PointLightShadowPassInitInfo {
            rhi: create_info.rhi,
        })?;

        m_tone_mapping_pass.initialize(&ToneMappingInitInfo {
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            rhi: create_info.rhi,
            input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD]
        })?;

        m_color_grading_pass.initialize(&ColorGradingPassInitInfo {
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            rhi: create_info.rhi,
            input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN],
        })?;

        m_fxaa_pass.initialize(&FXAAPassInitInfo {
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            rhi: create_info.rhi,
            input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD],
        })?;

        m_ui_pass.initialize(&UIPassInitInfo {
            rhi: create_info.rhi,
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            ctx: create_info.imgui_context,
            platform: create_info.imgui_platform,
        })?;

        m_combine_ui_pass.initialize(&CombineUIPassInitInfo {
            rhi: create_info.rhi,
            render_pass: *m_main_camera_pass.m_render_pass.get_render_pass(),
            scene_input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD],
            ui_input_attachment: m_main_camera_pass.m_render_pass.get_framebuffer_image_views()[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN],
        })?;

        m_pick_pass.initialize(&PickPassInitInfo {
            rhi: create_info.rhi,
            per_mesh_layout: descriptor_layouts[LayoutType::PerMesh as usize],
        })?;

        Ok(RenderPipeline {
            m_base: RefCell::new(RenderPipelineBase {
                m_rhi: Rc::downgrade(create_info.rhi),
                m_directional_light_pass,
                m_point_light_pass,
                m_main_camera_pass,  
                m_tone_mapping_pass,
                m_color_grading_pass,             
                m_fxaa_pass, 
                m_ui_pass,
                m_combine_ui_pass,
                m_pick_pass,
            })
        })
    }

    pub fn forward_render(&self, render_resource: &mut RenderResource) -> Result<()> {
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

            self.m_base.borrow().m_directional_light_pass.draw();
            self.m_base.borrow().m_point_light_pass.draw();

            self.m_base.borrow().m_main_camera_pass.draw_forward(
                &self.m_base.borrow().m_tone_mapping_pass,
                &self.m_base.borrow().m_color_grading_pass,
                &self.m_base.borrow().m_fxaa_pass,
                &self.m_base.borrow().m_ui_pass,
                &self.m_base.borrow().m_combine_ui_pass,
                rhi.get_current_swapchain_image_index()
            )?;

            let mut debugdraw_manager = RuntimeGlobalContext::get_debugdraw_manager().borrow_mut();
            debugdraw_manager.draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi))?;
        }
        Ok(())
    }

    pub fn deferred_render(&self, render_resource: &mut RenderResource) -> Result<()> {
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

            self.m_base.borrow().m_directional_light_pass.draw();
            self.m_base.borrow().m_point_light_pass.draw();

            self.m_base.borrow().m_main_camera_pass.draw(
                &self.m_base.borrow().m_tone_mapping_pass,
                &self.m_base.borrow().m_color_grading_pass,
                &self.m_base.borrow().m_fxaa_pass,
                &self.m_base.borrow().m_ui_pass,
                &self.m_base.borrow().m_combine_ui_pass,
                rhi.get_current_swapchain_image_index()
            )?;

            let mut debugdraw_manager = RuntimeGlobalContext::get_debugdraw_manager().borrow_mut();
            debugdraw_manager.draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|self.pass_update_after_recreate_swapchain(&rhi))?;
        }
        Ok(())
    }

    pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
        let pick_pass = &self.m_base.borrow().m_pick_pass;
        pick_pass.pick(picked_uv)   
    }
}

impl RenderPipeline {
    fn pass_update_after_recreate_swapchain(&self, rhi: &VulkanRHI) {
        self.m_base.borrow_mut().m_main_camera_pass.recreate_after_swapchain(rhi).unwrap();
        let image_views = self.m_base.borrow().m_main_camera_pass.m_render_pass.get_framebuffer_image_views();
        self.m_base.borrow_mut().m_tone_mapping_pass.update_after_framebuffer_recreate(
            rhi,
            image_views[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD],
        ).unwrap();
        self.m_base.borrow_mut().m_color_grading_pass.update_after_framebuffer_recreate(
            rhi,
            image_views[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN],
        ).unwrap();
        self.m_base.borrow_mut().m_fxaa_pass.update_after_framebuffer_recreate(
            rhi,
            image_views[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD]
        ).unwrap();
        self.m_base.borrow_mut().m_combine_ui_pass.update_after_framebuffer_recreate(
            rhi,
            image_views[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD],
            image_views[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN]
        ).unwrap();
        RuntimeGlobalContext::get_debugdraw_manager().borrow_mut().update_after_recreate_swap_chain(rhi);
    }
}