use vulkanalia::prelude::v1_0::*;
use crate::function::render::interface::rhi_struct::{RHIBuffer, RHIBufferView, RHICommandBuffer, RHICommandPool, RHIDescriptorPool, RHIDescriptorSet, RHIDescriptorSetLayout, RHIDevice, RHIDeviceMemory, RHIEvent, RHIFence, RHIFramebuffer, RHIImage, RHIImageView, RHIInstance, RHIPhysicalDevice, RHIPipeline, RHIPipelineCache, RHIPipelineLayout, RHIQueue, RHIRenderPass, RHISampler, RHISemaphore, RHIShader};

macro_rules! impl_vulkan_rhi_wrapper {
    ($struct_name:ident, $trait_name:ident, $field_type:ty) => {
        pub struct $struct_name {
            m_resource: $field_type,
        }

        impl $trait_name for $struct_name {}

        impl $struct_name {
            pub fn new(resource: $field_type) -> Self {
                Self {
                    m_resource: resource,
                }
            }
            pub fn set_resource(&mut self, res: $field_type) {
                self.m_resource = res;
            }
            pub fn get_resource(&self) -> $field_type {
                self.m_resource
            }
        }
    };
}

impl_vulkan_rhi_wrapper!(VulkanBuffer, RHIBuffer, vk::Buffer);
impl_vulkan_rhi_wrapper!(VulkanBufferView, RHIBufferView, vk::BufferView);
impl_vulkan_rhi_wrapper!(VulkanCommandBuffer, RHICommandBuffer, vk::CommandBuffer);
impl_vulkan_rhi_wrapper!(VulkanCommandPool, RHICommandPool, vk::CommandPool);
impl_vulkan_rhi_wrapper!(VulkanDescriptorPool, RHIDescriptorPool, vk::DescriptorPool);
impl_vulkan_rhi_wrapper!(VulkanDescriptorSet, RHIDescriptorSet, vk::DescriptorSet);
impl_vulkan_rhi_wrapper!(VulkanDescriptorSetLayout, RHIDescriptorSetLayout, vk::DescriptorSetLayout);
impl_vulkan_rhi_wrapper!(VulkanDevice, RHIDevice, vk::Device);
impl_vulkan_rhi_wrapper!(VulkanDeviceMemory, RHIDeviceMemory, vk::DeviceMemory);
impl_vulkan_rhi_wrapper!(VulkanEvent, RHIEvent, vk::Event);
impl_vulkan_rhi_wrapper!(VulkanFence, RHIFence, vk::Fence);
impl_vulkan_rhi_wrapper!(VulkanFramebuffer, RHIFramebuffer, vk::Framebuffer);
impl_vulkan_rhi_wrapper!(VulkanImage, RHIImage, vk::Image);
impl_vulkan_rhi_wrapper!(VulkanImageView, RHIImageView, vk::ImageView);
impl_vulkan_rhi_wrapper!(VulkanInstance, RHIInstance, vk::Instance);
impl_vulkan_rhi_wrapper!(VulkanQueue, RHIQueue, vk::Queue);
impl_vulkan_rhi_wrapper!(VulkanPhysicalDevice, RHIPhysicalDevice, vk::PhysicalDevice);
impl_vulkan_rhi_wrapper!(VulkanPipeline, RHIPipeline, vk::Pipeline);
impl_vulkan_rhi_wrapper!(VulkanPipelineCache, RHIPipelineCache, vk::PipelineCache);
impl_vulkan_rhi_wrapper!(VulkanPipelineLayout, RHIPipelineLayout, vk::PipelineLayout);
impl_vulkan_rhi_wrapper!(VulkanRenderPass, RHIRenderPass, vk::RenderPass);
impl_vulkan_rhi_wrapper!(VulkanSampler, RHISampler, vk::Sampler);
impl_vulkan_rhi_wrapper!(VulkanSemaphore, RHISemaphore, vk::Semaphore);
impl_vulkan_rhi_wrapper!(VulkanShader, RHIShader, vk::ShaderModule);

