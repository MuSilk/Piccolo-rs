use std::{any::Any};

use anyhow::Result;

use crate::runtime::function::render::{interface::rhi_struct::{RHIBuffer, RHICommandBuffer, RHIDepthImageDesc, RHIDescriptorPool, RHIDescriptorSet, RHIDescriptorSetAllocateInfo, RHIDescriptorSetLayout, RHIDescriptorSetLayoutCreateInfo, RHIDeviceMemory, RHIFramebuffer, RHIFramebufferCreateInfo, RHIGraphicsPipelineCreateInfo, RHIImage, RHIImageView, RHIPipeline, RHIPipelineLayout, RHIPipelineLayoutCreateInfo, RHIRect2D, RHIRenderPass, RHIRenderPassBeginInfo, RHIRenderPassCreateInfo, RHISampler, RHIShader, RHISwapChainDesc, RHIViewport, RHIWriteDescriptorSet}, render_type::{RHIBufferUsageFlags, RHIDefaultSamplerType, RHIDeviceSize, RHIFormat, RHIMemoryMapFlags, RHIMemoryPropertyFlags, RHIPipelineBindPoint, RHISubpassContents}, window_system::WindowSystem};

pub struct RHIInitInfo<'a> {
    pub window_system : &'a WindowSystem,
}

pub trait RHI: Any + Send + Sync {
    fn initialize(&mut self, info: RHIInitInfo) -> Result<()>;

    // allocate and create
    fn allocate_descriptor_sets(&self, allocate_info:&RHIDescriptorSetAllocateInfo) -> Result<Vec<Box<dyn RHIDescriptorSet>>>;
    fn create_swapchain(&mut self) -> Result<()>;
    fn create_swapchain_image_views(&mut self) -> Result<()>;
    fn get_or_create_default_sampler(&mut self, sampler_type: RHIDefaultSamplerType) -> Result<&Box<dyn RHISampler>>;
    fn create_shader_module(&self, data: &[u8]) -> Result<Box<dyn RHIShader>>;
    fn create_buffer(&self,size: RHIDeviceSize, usage: RHIBufferUsageFlags, properties: RHIMemoryPropertyFlags) -> Result<(Box<dyn RHIBuffer>, Box<dyn RHIDeviceMemory>)>;
    fn copy_buffer(&self, src: &Box<dyn RHIBuffer>, dst: &Box<dyn RHIBuffer>, src_offset: RHIDeviceSize, dst_offset: RHIDeviceSize, size: RHIDeviceSize) -> Result<()>;
    fn create_texture_image(&self, width: u32, height: u32, pixels: &[u8], format: RHIFormat, mip_levels: u32) -> Result<(Box<dyn RHIImage>, Box<dyn RHIDeviceMemory>, Box<dyn RHIImageView>)>;
    fn create_descriptor_set_layout(&self, create_info:&RHIDescriptorSetLayoutCreateInfo) -> Result<Box<dyn RHIDescriptorSetLayout>>;
    fn create_framebuffer(&self, create_info: &RHIFramebufferCreateInfo) -> Result<Box<dyn RHIFramebuffer>>;
    fn create_graphics_pipelines(&self, create_info:&[RHIGraphicsPipelineCreateInfo]) -> Result<Vec<Box<dyn RHIPipeline>>>;
    fn create_pipeline_layout(&self, create_info: &RHIPipelineLayoutCreateInfo) -> Result<Box<dyn RHIPipelineLayout>>;
    fn create_render_pass(&self, create_info: &RHIRenderPassCreateInfo) -> Result<Box<dyn RHIRenderPass>>;

    // command and command write
    fn cmd_begin_render_pass(&self, command_buffer: &Box<dyn RHICommandBuffer>, render_pass_begin_info: &RHIRenderPassBeginInfo, contents: RHISubpassContents);
    fn cmd_end_render_pass(&self, command_buffer: &Box<dyn RHICommandBuffer>);
    fn cmd_bind_pipeline(&self, command_buffer: &Box<dyn RHICommandBuffer>, pipeline_bind_point: RHIPipelineBindPoint, pipeline: &Box<dyn RHIPipeline>);
    fn cmd_set_viewport(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_viewport: u32, viewports: &[RHIViewport]);
    fn cmd_set_scissor(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_scissor: u32, scissors: &[RHIRect2D]);
    fn cmd_bind_vertex_buffers(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_binding: u32, buffers: &[&Box<dyn RHIBuffer>], offsets: &[RHIDeviceSize]);
    fn cmd_bind_descriptor_sets(&self, command_buffer: &Box<dyn RHICommandBuffer>, pipeline_bind_point: RHIPipelineBindPoint, layout: &Box<dyn RHIPipelineLayout>, first_set: u32, descriptor_sets: &[&Box<dyn RHIDescriptorSet>], dynamic_offsets: &[u32]);
    
    fn cmd_draw(&self, command_buffer: &Box<dyn RHICommandBuffer>,vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32);
    fn update_descriptor_sets(&self, writes: &[RHIWriteDescriptorSet]) -> Result<()>;

    // query
    fn get_current_command_buffer(&self) -> &Box<dyn RHICommandBuffer>;
    fn get_descriptor_pool(&self) -> Result<&Box<dyn RHIDescriptorPool>>;
    fn get_swap_chain_info(&'_ self) -> RHISwapChainDesc<'_>;
    fn get_depth_image_info(&'_ self) -> RHIDepthImageDesc<'_>;
    fn get_max_frames_in_flight(&self) -> u8;
    fn get_current_frame_index(&self) -> u8;

    // command write
    fn begin_single_time_commands(&self) -> Result<Box<dyn RHICommandBuffer>>;
    fn end_single_time_commands(&self, command_buffer: Box<dyn RHICommandBuffer>) -> Result<()>;
    fn push_event(&self, command_buffer: &Box<dyn RHICommandBuffer>, event_name: &str, color: [f32; 4]);
    fn pop_event(&self, command_buffer: &Box<dyn RHICommandBuffer>);

    // destory
    fn destroy_shader_module(&self, shader: Box<dyn RHIShader>);
    fn destroy_image_view(&self, image_view: Box<dyn RHIImageView>);
    fn destroy_image(&self, image: Box<dyn RHIImage>);
    fn destroy_framebuffer(&self, framebuffer: Box<dyn RHIFramebuffer>);
    fn destroy_buffer(&self, buffer: Box<dyn RHIBuffer>);

    // memory
    fn free_memory(&self, memory: Box<dyn RHIDeviceMemory>);
    fn map_memory(&self, memory: &Box<dyn RHIDeviceMemory>, offset: RHIDeviceSize, size: RHIDeviceSize, flags: RHIMemoryMapFlags) -> Result<*mut std::ffi::c_void>;
    fn unmap_memory(&self, memory: &Box<dyn RHIDeviceMemory>);

    // semaphore

    
}
impl dyn RHI {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}