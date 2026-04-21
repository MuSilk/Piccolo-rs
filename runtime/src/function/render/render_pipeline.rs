use crate::{
    core::math::vector2::Vector2,
    function::{
        render::{
            interface::vulkan::vulkan_rhi::VulkanRHI,
            passes::{
                directional_light_pass::{
                    DirectionalLightShadowPass, DirectionalLightShadowPassInitInfo,
                },
                main_camera_pass::{LayoutType, MainCameraPass, MainCameraPassInitInfo},
                pick_pass::{PickPass, PickPassInitInfo},
                point_light_pass::{PointLightShadowPass, PointLightShadowPassInitInfo},
            },
            render_resource::RenderResource,
        },
        ui::ui2::UiRuntime,
    },
};
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub struct RenderPipelineCreateInfo<'a> {
    pub rhi: &'a VulkanRHI,
    pub render_resource: &'a RenderResource,
    pub enable_fxaa: bool,
}

pub struct RenderPipeline {
    m_directional_light_pass: DirectionalLightShadowPass,
    m_point_light_pass: PointLightShadowPass,
    m_main_camera_pass: MainCameraPass,
    m_pick_pass: PickPass,
}

impl RenderPipeline {
    pub fn create(create_info: &RenderPipelineCreateInfo) -> Result<Self> {
        let mut m_directional_light_pass = DirectionalLightShadowPass::default();
        let mut m_point_light_pass = PointLightShadowPass::default();
        let mut m_main_camera_pass = MainCameraPass::default();
        let mut m_pick_pass = PickPass::default();

        let global_render_resource = &create_info.render_resource.m_global_render_resource;

        m_directional_light_pass.initialize(&DirectionalLightShadowPassInitInfo {
            rhi: create_info.rhi,
            global_render_resource: &global_render_resource,
        })?;

        m_point_light_pass.initialize(&PointLightShadowPassInitInfo {
            rhi: create_info.rhi,
            global_render_resource: &global_render_resource,
        })?;

        m_main_camera_pass.m_directional_light_shadow_color_image_view = m_directional_light_pass
            .m_render_pass
            .m_framebuffer
            .attachments[0]
            .view;

        m_main_camera_pass.m_point_light_shadow_color_image_view =
            m_point_light_pass.m_render_pass.m_framebuffer.attachments[0].view;

        m_main_camera_pass.initialize(&MainCameraPassInitInfo {
            rhi: create_info.rhi,
            enable_fxaa: create_info.enable_fxaa,
            global_render_resource: &global_render_resource,
        })?;

        m_directional_light_pass.set_per_mesh_layout(
            m_main_camera_pass.get_descriptor_set_layouts(LayoutType::PerMesh),
        );
        m_point_light_pass.set_per_mesh_layout(
            m_main_camera_pass.get_descriptor_set_layouts(LayoutType::PerMesh),
        );

        m_directional_light_pass.post_initialize(&DirectionalLightShadowPassInitInfo {
            rhi: create_info.rhi,
            global_render_resource: &global_render_resource,
        })?;
        m_point_light_pass.post_initialize(&PointLightShadowPassInitInfo {
            rhi: create_info.rhi,
            global_render_resource: &global_render_resource,
        })?;

        m_pick_pass.initialize(&PickPassInitInfo {
            rhi: create_info.rhi,
            per_mesh_layout: m_main_camera_pass.get_descriptor_set_layouts(LayoutType::PerMesh),
            global_render_resource: &global_render_resource,
        })?;

        Ok(RenderPipeline {
            m_directional_light_pass,
            m_point_light_pass,
            m_main_camera_pass,
            m_pick_pass,
        })
    }

    pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
        // let pick_pass = &self.m_base.borrow().m_pick_pass;
        // pick_pass.pick(picked_uv)
        0
    }

    pub fn prepare_pass_data(&mut self, rhi: &VulkanRHI, render_resource: &RenderResource) {
        self.m_directional_light_pass
            .prepare_pass_data(render_resource);
        self.m_point_light_pass.prepare_pass_data(render_resource);
        self.m_main_camera_pass.prepare_pass_data(render_resource);
        self.m_pick_pass.prepare_pass_data(rhi, render_resource);
    }

    pub fn destroy(&self, rhi: &VulkanRHI) {
        self.m_main_camera_pass.destroy(rhi);
    }

    pub fn draw(&self, rhi: &VulkanRHI, ui_runtime: &UiRuntime, forward_draw: bool) {
        self.m_directional_light_pass.draw(rhi);
        self.m_point_light_pass.draw(rhi);

        self.m_main_camera_pass
            .draw(
                rhi,
                ui_runtime,
                rhi.get_current_swapchain_image_index(),
                forward_draw,
            )
            .unwrap();
    }

    pub fn recreate_after_swapchain(&mut self, rhi: &VulkanRHI) {
        self.m_main_camera_pass
            .recreate_after_swapchain(rhi)
            .unwrap();
    }

    pub fn get_descriptor_set_layouts(&self, layout_type: LayoutType) -> vk::DescriptorSetLayout {
        self.m_main_camera_pass
            .get_descriptor_set_layouts(layout_type)
    }
}
