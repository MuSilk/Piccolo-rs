pub mod pbr_pipeline;
pub mod ui_pipeline;

use crate::function::{
    render::{
        interface::vulkan::vulkan_rhi::VulkanRHI,
        render_pass::DescriptorLayoutRegistry,
        render_resource::{GlobalRenderResource, RenderResource},
        render_scene::RenderScene,
    },
    ui::ui2::UiRuntime,
};

pub trait RenderPipelineTrait {
    fn prepare_pass_data(&mut self, rhi: &VulkanRHI, render_resource: &RenderResource);
    fn supports_debugdraw(&self) -> bool;
    fn destroy(&self, rhi: &VulkanRHI);
    fn draw(
        &self,
        rhi: &VulkanRHI,
        render_scene: &RenderScene,
        render_resource: &mut GlobalRenderResource,
        ui_runtime: &UiRuntime,
    );
    fn recreate_after_swapchain(&mut self, rhi: &VulkanRHI, render_resource: &GlobalRenderResource);
}

pub struct RenderPipelineCreateInfo<'a> {
    pub rhi: &'a VulkanRHI,
    pub render_resource: &'a RenderResource,
    pub descriptor_layout_registry: &'a DescriptorLayoutRegistry,
    pub enable_fxaa: bool,
}
