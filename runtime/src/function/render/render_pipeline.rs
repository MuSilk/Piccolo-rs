use crate::function::{
    render::{
        interface::vulkan::vulkan_rhi::VulkanRHI,
        passes::{
            directional_light_pass::{
                DirectionalLightShadowPass, DirectionalLightShadowPassInitInfo,
            },
            main_camera_pass::{MainCameraPass, MainCameraPassInitInfo, PerMeshDescriptorLayout},
            pick_pass::{PickPass, PickPassInitInfo},
            point_light_pass::{PointLightShadowPass, PointLightShadowPassInitInfo},
        },
        render_pass::DescriptorLayoutRegistry,
        render_resource::{GlobalRenderResource, RenderResource},
        render_scene::RenderScene,
        render_type::RenderPipelineType,
    },
    ui::ui2::UiRuntime,
};
use anyhow::Result;

pub struct RenderPipelineCreateInfo<'a> {
    pub rhi: &'a VulkanRHI,
    pub render_resource: &'a RenderResource,
    pub descriptor_layout_registry: &'a DescriptorLayoutRegistry,
    pub enable_fxaa: bool,
}

pub struct RenderPipeline {
    m_render_pipeline_type: RenderPipelineType,
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
            descriptor_layout_registry: &create_info.descriptor_layout_registry,
            global_render_resource: &global_render_resource,
        })?;

        m_point_light_pass.initialize(&PointLightShadowPassInitInfo {
            rhi: create_info.rhi,
            descriptor_layout_manager: &create_info.descriptor_layout_registry,
            global_render_resource: &global_render_resource,
        })?;

        m_main_camera_pass.m_directional_light_shadow_color_image_view = m_directional_light_pass
            .m_directional_light_shadow_attachment
            .view;

        m_main_camera_pass.m_point_light_shadow_color_image_view =
            m_point_light_pass.m_point_light_shadow_attachment.view;

        m_main_camera_pass.initialize(&MainCameraPassInitInfo {
            rhi: create_info.rhi,
            enable_fxaa: create_info.enable_fxaa,
            global_render_resource: &global_render_resource,
            descriptor_layout_manager: &create_info.descriptor_layout_registry,
        })?;

        m_pick_pass.initialize(&PickPassInitInfo {
            rhi: create_info.rhi,
            per_mesh_layout: create_info
                .descriptor_layout_registry
                .acquire::<PerMeshDescriptorLayout>(create_info.rhi)?,
            global_render_resource: &global_render_resource,
        })?;

        Ok(RenderPipeline {
            m_render_pipeline_type: RenderPipelineType::DeferredPipeline,
            m_directional_light_pass,
            m_point_light_pass,
            m_main_camera_pass,
            m_pick_pass,
        })
    }

    // pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
    //     self.m_pick_pass.pick(picked_uv)
    // }

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

    pub fn draw(
        &self,
        rhi: &VulkanRHI,
        render_scene: &RenderScene,
        render_resource: &mut GlobalRenderResource,
        ui_runtime: &UiRuntime,
    ) {
        self.m_directional_light_pass
            .draw(rhi, render_scene, render_resource);
        self.m_point_light_pass.draw(rhi);
        self.m_main_camera_pass
            .draw(
                rhi,
                render_scene,
                render_resource,
                ui_runtime,
                match self.m_render_pipeline_type {
                    RenderPipelineType::ForwardPipeline => true,
                    RenderPipelineType::DeferredPipeline => false,
                },
            )
            .unwrap();
    }

    pub fn recreate_after_swapchain(
        &mut self,
        rhi: &VulkanRHI,
        render_resource: &GlobalRenderResource,
    ) {
        self.m_main_camera_pass
            .recreate_after_swapchain(rhi, render_resource)
            .unwrap();
    }
}
