use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{core::math::vector2::Vector2, function::{render::{debugdraw::debug_draw_manager::DebugDrawManager, interface::vulkan::vulkan_rhi::VulkanRHI, passes::{directional_light_pass::{DirectionalLightShadowPass, DirectionalLightShadowPassInitInfo}, main_camera_pass::{LayoutType, MainCameraPass, MainCameraPassInitInfo}, pick_pass::{PickPass, PickPassInitInfo}, point_light_pass::{PointLightShadowPass, PointLightShadowPassInitInfo}}, render_pass_base::RenderPassCommonInfo, render_pipeline_base::{RenderPipelineBase, RenderPipelineCreateInfo}, render_resource::RenderResource}, ui::ui2::UiRuntime}};


pub struct RenderPipeline {
    pub m_base : RefCell<RenderPipelineBase>
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        let mut m_directional_light_pass = DirectionalLightShadowPass::default();
        let mut m_point_light_pass = PointLightShadowPass::default();
        let mut m_main_camera_pass = MainCameraPass::default();
        let mut m_pick_pass = PickPass::default();

        let common_info = RenderPassCommonInfo {
            rhi: create_info.rhi,
            render_resource: create_info.render_resource,
        };
        m_directional_light_pass.m_render_pass.set_common_info(&common_info);
        m_point_light_pass.m_render_pass.set_common_info(&common_info);
        m_main_camera_pass.set_common_info(&common_info);
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
            rhi: &create_info.rhi.borrow(),
            config_manager: create_info.config_manager,
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
                m_pick_pass,
            })
        })
    }

    pub fn render(
        &self, 
        debugdraw_manager: &RefCell<DebugDrawManager>,
        ui_runtime: &RefCell<UiRuntime>,
        render_resource: &mut RenderResource,
        forward_draw: bool
    ) -> Result<()> {
        let rhi = self.m_base.borrow().m_rhi.upgrade().unwrap();
        render_resource.reset_ring_buffer_offset(rhi.borrow().get_current_frame_index());
        {
            let mut rhi = rhi.borrow_mut();
            rhi.wait_for_fence()?;
            rhi.reset_command_pool()?;
            if rhi.prepare_before_pass(
                &|rhi: &VulkanRHI|
                self.pass_update_after_recreate_swapchain(&debugdraw_manager, &rhi)
            )? {
                return Ok(());
            }
        }
        {
            let rhi = rhi.borrow();

            self.m_base.borrow().m_directional_light_pass.draw();
            self.m_base.borrow().m_point_light_pass.draw();

            self.m_base.borrow().m_main_camera_pass.draw(
                ui_runtime,
                rhi.get_current_swapchain_image_index(),
                forward_draw
            )?;

            debugdraw_manager.borrow_mut().draw(rhi.get_current_swapchain_image_index())?;
        }
        {
            let mut rhi = rhi.borrow_mut();
            rhi.submit_rendering(&|rhi: &VulkanRHI|
                self.pass_update_after_recreate_swapchain(&debugdraw_manager, &rhi))?;
        }
        Ok(())
    }

    pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
        // let pick_pass = &self.m_base.borrow().m_pick_pass;
        // pick_pass.pick(picked_uv) 
        0  
    }
}

impl RenderPipeline {
    fn pass_update_after_recreate_swapchain(
        &self, 
        debugdraw_manager: &RefCell<DebugDrawManager>,
        rhi: &VulkanRHI
    ) {
        self.m_base.borrow_mut().m_main_camera_pass.recreate_after_swapchain(rhi).unwrap();
        debugdraw_manager.borrow_mut().update_after_recreate_swap_chain(rhi);
    }
}