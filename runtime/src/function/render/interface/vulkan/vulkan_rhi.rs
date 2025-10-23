use std::{cell::{OnceCell, RefCell}, collections::{HashMap, HashSet}, ffi::CStr, fmt::Debug, os::raw::c_void, rc::{Rc, Weak}};

use anyhow::{anyhow, Result};
use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, prelude::v1_0::*, vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension}, window::{self as vk_window}, Version};
use winit::window::Window;
use log::*;

use crate::function::render::{interface::{rhi::{self, RHICreateInfo}, rhi_struct::{QueueFamilyIndices, RHIDepthImageDesc, RHISwapChainDesc, SwapChainSupportDetails}, vulkan::vulkan_util::{self, create_image_view}}, render_type::RHISamplerType};

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
pub const K_MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct VulkanRHI {
    _m_entry: Entry,
    pub m_instance: Instance,
    pub m_data: VulkanRHIData,
    pub m_device: Device,
    m_current_frame_index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VulkanRHIData {
    // Pipeline
    m_images_in_flight: Vec<vk::Fence>,

    m_graphics_queue: vk::Queue,

    m_swapchain_image_format: vk::Format,
    m_swapchain_image_views: Vec<vk::ImageView>,
    m_swapchain_extent: vk::Extent2D,
    pub m_viewport: vk::Viewport,
    pub m_scissor: vk::Rect2D,

    m_depth_image_format: vk::Format,
    m_depth_image_view: vk::ImageView,

    m_descriptor_pool: vk::DescriptorPool,

    m_command_pool: vk::CommandPool,

    m_command_buffers: [vk::CommandBuffer; K_MAX_FRAMES_IN_FLIGHT],
    m_current_command_buffer: vk::CommandBuffer,

    m_queue_indices: QueueFamilyIndices,

    m_window: Weak<Window>,
    m_surface: vk::SurfaceKHR,
    pub m_physical_device: vk::PhysicalDevice,
    m_present_queue: vk::Queue,

    m_swapchain: vk::SwapchainKHR,
    m_swapchain_images: Vec<vk::Image>,

    m_depth_image: vk::Image,
    m_depth_image_memory: vk::DeviceMemory,

    // // m_assert_allocator: Option<gpu_allocator::vulkan::Allocator>,

    m_command_pools: [vk::CommandPool; K_MAX_FRAMES_IN_FLIGHT],
    m_image_available_for_render_semaphores: [vk::Semaphore; K_MAX_FRAMES_IN_FLIGHT as usize],
    m_image_finished_for_presentation_semaphores: [vk::Semaphore; K_MAX_FRAMES_IN_FLIGHT as usize],
    // m_image_available_for_texturescopy_semaphores: Option<[Box<dyn RHISemaphore>; Self::K_MAX_FRAMES_IN_FLIGHT as usize]>,
    m_is_frame_in_flight_fences: [vk::Fence; K_MAX_FRAMES_IN_FLIGHT as usize],

    m_current_swapchain_image_index: usize,

    m_validation_layers: Vec<vk::ExtensionName>,
    m_vulkan_api_version: u32,

    m_device_extensions: Vec<vk::ExtensionName>,

    m_linear_sampler: OnceCell<vk::Sampler>,
    m_nearest_sampler: OnceCell<vk::Sampler>,
    m_mipmap_sampler_map: RefCell<HashMap<(u32, RHISamplerType), vk::Sampler>>,

    m_enable_validation_layers: bool,
    m_enable_debug_utils_label: bool,

    m_max_vertex_blending_mesh_count: u32,
    m_max_material_count: u32,

    m_debug_messenger: vk::DebugUtilsMessengerEXT,   
}

impl VulkanRHI {

    pub fn create(info: &RHICreateInfo) -> Result<Self> {

        let mut data = VulkanRHIData::default();
        data.m_max_material_count = 256;
        data.m_max_vertex_blending_mesh_count = 256;

        let window_system = info.window_system;
        data.m_window = Rc::downgrade(window_system.get_window());
        let window_size = window_system.get_window_size();

        data.m_viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: window_size.0 as f32,
            height: window_size.1 as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        data.m_scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: window_size.0, height: window_size.1 },
        };

        let window = info.window_system.get_window();

        let entry = unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            Entry::new(loader).map_err(|b| anyhow!("{}", b))?
        };

        data.m_validation_layers = vec![VALIDATION_LAYER];
        data.m_device_extensions = DEVICE_EXTENSIONS.to_vec();

        if cfg!(debug_assertions) {
            data.m_enable_validation_layers = true;
            data.m_enable_debug_utils_label = true;
        }
        else{
            data.m_enable_validation_layers = false;
            data.m_enable_debug_utils_label = false;
        }

        let instance = create_instance(window, &entry, &mut data)?;
        initialize_debug_messenger(&instance, &mut data)?;
        create_window_surface(&instance, window, &mut data)?;
        initial_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        create_command_pool(&device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;
        create_descriptor_pool(&device, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;

        Ok(Self {
            _m_entry: entry,
            m_instance: instance,
            m_data: data,
            m_device: device,
            m_current_frame_index: 0,
        })
    }

    pub fn destroy(&mut self) {
        unsafe{
            if let Some(sampler) = self.m_data.m_linear_sampler.get() {
                self.m_device.destroy_sampler(*sampler, None);
            }
            if let Some(sampler) = self.m_data.m_nearest_sampler.get() {
                self.m_device.destroy_sampler(*sampler, None);
            }

            self.m_device.destroy_image_view(self.m_data.m_depth_image_view, None);
            self.m_device.destroy_image(self.m_data.m_depth_image, None);
            self.m_device.free_memory(self.m_data.m_depth_image_memory, None);

            self.m_data.m_swapchain_image_views.iter().for_each(|v| self.m_device.destroy_image_view(*v, None));
            self.m_device.destroy_swapchain_khr(self.m_data.m_swapchain, None);
            self.m_device.destroy_descriptor_pool(self.m_data.m_descriptor_pool, None);
            self.m_data.m_is_frame_in_flight_fences.iter().for_each(|f| self.m_device.destroy_fence(*f, None));
            self.m_data.m_image_finished_for_presentation_semaphores.iter().for_each(|s| self.m_device.destroy_semaphore(*s, None));
            self.m_data.m_image_available_for_render_semaphores.iter().for_each(|s| self.m_device.destroy_semaphore(*s, None));
            self.m_data.m_command_pools.iter().for_each(|p| self.m_device.destroy_command_pool(*p, None));
            self.m_device.destroy_command_pool(self.m_data.m_command_pool, None);
            self.m_device.destroy_device(None);
            self.m_instance.destroy_surface_khr(self.m_data.m_surface, None);

            if VALIDATION_ENABLED {
                self.m_instance.destroy_debug_utils_messenger_ext(self.m_data.m_debug_messenger, None);
            }

            self.m_instance.destroy_instance(None);
        }
    }

    pub fn prepare_context(&mut self) {
        let command_buffer: vk::CommandBuffer = self.m_data.m_command_buffers[self.m_current_frame_index];
        self.m_data.m_current_command_buffer = command_buffer;
    }

    pub fn allocate_descriptor_sets(&self, allocate_info:&vk::DescriptorSetAllocateInfo) -> Result<Vec<vk::DescriptorSet>> {
        let descriptor_sets = unsafe{
            self.m_device.allocate_descriptor_sets(&allocate_info)?
        };
        Ok(descriptor_sets)
    }

    fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        unsafe{
            self.m_device.device_wait_idle()?;

            let window_size = window.inner_size();

            self.m_data.m_viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: window_size.width as f32,
                height: window_size.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            self.m_data.m_scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: window_size.width, height: window_size.height },
            };

            self.m_device.destroy_image_view(self.m_data.m_depth_image_view, None);
            self.m_device.destroy_image(self.m_data.m_depth_image, None);
            self.m_device.free_memory(self.m_data.m_depth_image_memory, None);
            self.m_data.m_swapchain_image_views.iter().for_each(|v| self.m_device.destroy_image_view(*v, None));
            self.m_device.destroy_swapchain_khr(self.m_data.m_swapchain, None);
            create_swapchain(window, &self.m_instance, &self.m_device, &mut self.m_data)?;
            create_swapchain_image_views(&self.m_device, &mut self.m_data)?;
            create_depth_objects(&self.m_instance, &self.m_device, &mut self.m_data)?;
            self.m_data.m_images_in_flight.resize(self.m_data.m_swapchain_images.len(), vk::Fence::null());
        }
        Ok(())
    }

    pub fn get_or_create_default_sampler(&self, sampler_type: RHISamplerType) -> Result<&vk::Sampler> {
        match sampler_type {
            RHISamplerType::Linear => {
                Ok(self.m_data.m_linear_sampler.get_or_init(|| {
                    let properties = unsafe{self.m_instance.get_physical_device_properties(self.m_data.m_physical_device)};
                    let create_info = vk::SamplerCreateInfo::builder()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .mip_lod_bias(0.0)
                        .anisotropy_enable(false)
                        .max_anisotropy(properties.limits.max_sampler_anisotropy)
                        .compare_enable(false)
                        .compare_op(vk::CompareOp::ALWAYS)
                        .min_lod(0.0)
                        .max_lod(8.0)
                        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                        .unnormalized_coordinates(false);

                    unsafe {
                        self.m_device.create_sampler(&create_info, None).unwrap()
                    }
                }))
            }
            RHISamplerType::Nearest => {
                Ok(self.m_data.m_nearest_sampler.get_or_init(||{
                    let properties = unsafe{self.m_instance.get_physical_device_properties(self.m_data.m_physical_device)};
                    let create_info = vk::SamplerCreateInfo::builder()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .mip_lod_bias(0.0)
                        .anisotropy_enable(false)
                        .max_anisotropy(properties.limits.max_sampler_anisotropy)
                        .compare_enable(false)
                        .compare_op(vk::CompareOp::ALWAYS)
                        .min_lod(0.0)
                        .max_lod(8.0)
                        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                        .unnormalized_coordinates(false);

                    unsafe {
                        self.m_device.create_sampler(&create_info, None).unwrap()
                    }
                }))
            }
        }
    }

    pub fn get_or_create_mipmap_sampler(&self, width: u32, height: u32, sampler_type: RHISamplerType) -> Result<vk::Sampler> {
        let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;
        if let Some(sampler) = self.m_data.m_mipmap_sampler_map.borrow().get(&(mip_levels, sampler_type)) {
            return Ok(*sampler);
        }

        let physical_device_properties = unsafe {
            self.m_instance.get_physical_device_properties(self.m_data.m_physical_device)
        };
        let filter = match sampler_type {
            RHISamplerType::Nearest => vk::Filter::NEAREST,
            RHISamplerType::Linear => vk::Filter::LINEAR,
        };
        let anisotropy_enable = sampler_type == RHISamplerType::Linear;
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(filter)
            .min_filter(filter)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(anisotropy_enable)
            .max_anisotropy(physical_device_properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod((mip_levels - 1) as f32);

        let sampler = unsafe {
            self.m_device.create_sampler(&sampler_info, None)?
        };
        
        self.m_data.m_mipmap_sampler_map.borrow_mut().insert((mip_levels, sampler_type), sampler);
        Ok(sampler)
    }

    pub fn create_shader_module(&self, data: &[u8]) -> Result<vk::ShaderModule> {
        let shader_module = vulkan_util::create_shader_module(&self.m_device, data)?;
        Ok(shader_module)
    }

    pub fn create_buffer(&self,size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) 
        -> Result<(vk::Buffer, vk::DeviceMemory)> 
    {
        let (buffer,memory) = vulkan_util::create_buffer(
            &self.m_instance, 
            &self.m_device, 
            self.m_data.m_physical_device, size, 
            usage,
            properties,
        )?;
        Ok((buffer, memory))
    }

    pub fn copy_buffer(&self, src_buffer: vk::Buffer, dst_buffer: vk::Buffer, src_offset: vk::DeviceSize, dst_offset: vk::DeviceSize, size: vk::DeviceSize) -> Result<()> {
        vulkan_util::copy_buffer(
            self,
            &self.m_device,
            src_buffer,
            dst_buffer,
            src_offset,
            dst_offset,
            size,
        )?;

        Ok(())
    }

    pub fn create_cube_map(&self, width: u32, height: u32, pixels: &[&[u8]; 6], format: vk::Format, mip_levels: u32) 
        -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> 
    {    
        Ok(vulkan_util::create_cube_map(self, width, height, pixels, format, mip_levels)?)
    }
    
    pub fn create_image(
        &self, width: u32, height: u32, format: vk::Format, 
        tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags,
        flags: vk::ImageCreateFlags, array_layers: u32, mip_levels: u32,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        Ok(vulkan_util::create_image(
            &self.m_instance, &self.m_device, self.m_data.m_physical_device,
            width, height, format, 
            tiling, usage, properties,
            flags,
            array_layers,
            mip_levels,
        )?)
    }
    
    pub fn create_image_view(
        &self, image: vk::Image, format: vk::Format, 
        aspect_flags: vk::ImageAspectFlags, 
        view_type: vk::ImageViewType,
        layout_count: u32,
        mip_levels: u32
    ) -> Result<vk::ImageView> {
        Ok(
            vulkan_util::create_image_view(
                &self.m_device, image, format, aspect_flags, view_type , layout_count, mip_levels,
            )?
        )
    }

    //mip_levels: 0 means auto
    pub fn create_texture_image(&self, width: u32, height: u32, pixels: &[u8], format: vk::Format, mip_levels: u32) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        Ok(vulkan_util::create_texture_image(
            self,
            width,
            height,
            pixels,
            vk::Format::from_raw(format.as_raw()),
            mip_levels,
        )?)
    }

    pub fn create_descriptor_set_layout(&self, create_info:&vk::DescriptorSetLayoutCreateInfo) -> Result<vk::DescriptorSetLayout>{
        Ok(unsafe {
            self.m_device.create_descriptor_set_layout(&create_info, None)?
        })
    }

    pub fn create_framebuffer(&self, create_info: &vk::FramebufferCreateInfo) -> Result<vk::Framebuffer> {
        Ok(unsafe {
            self.m_device.create_framebuffer(create_info, None)?
        })
    }
    
    pub fn create_graphics_pipelines(
        &self,
        pipeline_cache: vk::PipelineCache, 
        create_info:&[vk::GraphicsPipelineCreateInfo]) 
    -> Result<Vec<vk::Pipeline>> {
        Ok(unsafe { 
            self.m_device.create_graphics_pipelines(pipeline_cache, create_info, None)?.0 
        })
    }

    pub fn create_pipeline_layout(&self, create_info: &vk::PipelineLayoutCreateInfo) -> Result<vk::PipelineLayout> {
        Ok(unsafe{self.m_device.create_pipeline_layout(create_info, None)?})
    }

    pub fn create_render_pass(&self, create_info: &vk::RenderPassCreateInfo) -> Result<vk::RenderPass> {
        Ok(unsafe{self.m_device.create_render_pass(create_info, None)?})
    }
    
    pub fn create_sampler(&self, create_info: &vk::SamplerCreateInfo) -> Result<vk::Sampler> {
        Ok(unsafe{self.m_device.create_sampler(create_info, None)?})
    }

    pub fn cmd_begin_render_pass(&self, command_buffer: vk::CommandBuffer, begin_info: &vk::RenderPassBeginInfo, contents: vk::SubpassContents) {
        unsafe{
            self.m_device.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::from_raw(contents.as_raw()));
        }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe{
            self.m_device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn cmd_next_subpass(&self, command_buffer: vk::CommandBuffer, contents: vk::SubpassContents) {
        unsafe{
            self.m_device.cmd_next_subpass(command_buffer, contents);
        }
    }
    
    pub fn cmd_bind_pipeline(&self, command_buffer: vk::CommandBuffer, pipeline_bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe{
            self.m_device.cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline);
        }
    }
    
    pub fn cmd_bind_vertex_buffers(&self, command_buffer: vk::CommandBuffer, first_binding: u32, buffers: &[vk::Buffer], offsets: &[vk::DeviceSize]) {
        unsafe {
            self.m_device.cmd_bind_vertex_buffers(command_buffer, first_binding, buffers, offsets);
        }
    }

    pub fn cmd_bind_index_buffer(&self, command_buffer: vk::CommandBuffer, buffer: vk::Buffer, offset: vk::DeviceSize, index_type: vk::IndexType) {
        unsafe {
            self.m_device.cmd_bind_index_buffer(command_buffer, buffer, offset, index_type);
        }
    }
    
    pub fn cmd_bind_descriptor_sets(&self, command_buffer: vk::CommandBuffer, pipeline_bind_point: vk::PipelineBindPoint, layout: vk::PipelineLayout, first_set: u32, descriptor_sets: &[vk::DescriptorSet], dynamic_offsets: &[u32]) {
        unsafe {
            self.m_device.cmd_bind_descriptor_sets(
                command_buffer, 
                pipeline_bind_point, 
                layout, 
                first_set, 
                descriptor_sets, 
                dynamic_offsets,
            );
        }
    }
    
    pub fn cmd_push_constants(&self, command_buffer: vk::CommandBuffer, layout: vk::PipelineLayout, stage_flags: vk::ShaderStageFlags, offset: u32, values: &[u8]) {
        unsafe {
            self.m_device.cmd_push_constants(
                command_buffer, 
                layout, 
                stage_flags, 
                offset, 
                values,
            );
        }
    }

    pub fn cmd_set_viewport(&self, command_buffer: vk::CommandBuffer, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self.m_device.cmd_set_viewport(command_buffer, first_viewport, viewports);
        }
    }

    pub fn cmd_set_scissor(&self, command_buffer: vk::CommandBuffer, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self.m_device.cmd_set_scissor(command_buffer, first_scissor, scissors);
        }
    }

    pub fn cmd_draw(&self, command_buffer: vk::CommandBuffer, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32){
        unsafe {
            self.m_device.cmd_draw(
                command_buffer, 
                vertex_count, 
                instance_count, 
                first_vertex, 
                first_instance,
            );
        }
    }

    pub fn cmd_clear_attachments(&self, command_buffer: vk::CommandBuffer, clear_attachments: &[vk::ClearAttachment], clear_rects: &[vk::ClearRect]) {
        unsafe {
            self.m_device.cmd_clear_attachments(
                command_buffer, 
                clear_attachments, 
                clear_rects,
            );
        }
    }

    pub fn cmd_draw_indexed(&self, command_buffer: vk::CommandBuffer, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32){
        unsafe {
            self.m_device.cmd_draw_indexed(
                command_buffer, 
                index_count, 
                instance_count, 
                first_index, 
                vertex_offset, 
                first_instance,
            );
        }
    }

    pub fn update_descriptor_sets(&self, writes: &[vk::WriteDescriptorSet]) -> Result<()> {

        unsafe {
            self.m_device.update_descriptor_sets(&writes, &[] as &[vk::CopyDescriptorSet]);
        }

        Ok(())
    }

    pub fn reset_command_pool(&self) -> Result<()> {
        unsafe {
            self.m_device.reset_command_pool(self.m_data.m_command_pools[self.m_current_frame_index], vk::CommandPoolResetFlags::empty())?;
        }
        Ok(())
    }

    pub fn wait_for_fence(&self) -> Result<()>{
        unsafe {
            self.m_device.wait_for_fences(&[self.m_data.m_is_frame_in_flight_fences[self.m_current_frame_index]], true, std::u64::MAX)?;
        }
        Ok(())
    }
    
    pub fn get_physical_device_properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.m_instance.get_physical_device_properties(self.m_data.m_physical_device)
        }
    }

    pub fn wait_idle(&self) -> Result<()>{
        unsafe {
            self.m_device.device_wait_idle()?;
        }
        Ok(())
    }
    
    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        self.m_data.m_current_command_buffer
    }
    
    pub fn get_descriptor_pool(&self) -> vk::DescriptorPool {
        self.m_data.m_descriptor_pool
    }

    pub fn get_swapchain_info(&'_ self) -> RHISwapChainDesc<'_> {
        RHISwapChainDesc{
            extent: self.m_data.m_swapchain_extent,
            image_format: self.m_data.m_swapchain_image_format,
            viewport: &self.m_data.m_viewport,
            scissor: &self.m_data.m_scissor,
            image_views: &self.m_data.m_swapchain_image_views,
        }
    }

    pub fn get_depth_image_info(&'_ self) -> RHIDepthImageDesc<'_> {
        RHIDepthImageDesc {
            image: &self.m_data.m_depth_image,
            image_view: &self.m_data.m_depth_image_view,
            format: self.m_data.m_depth_image_format,
        }
    }
    
    pub const fn get_max_frames_in_flight() -> usize {
        K_MAX_FRAMES_IN_FLIGHT
    }

    pub fn get_current_frame_index(&self) -> usize {
        self.m_current_frame_index
    }

    pub fn get_current_swapchain_image_index(&self) -> usize {
        self.m_data.m_current_swapchain_image_index
    }

    pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.m_data.m_command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { self.m_device.allocate_command_buffers(&allocate_info)?[0] };

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.m_device.begin_command_buffer(command_buffer, &begin_info)?
        }
        Ok(command_buffer)
    }

    pub fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer) -> Result<()> {
        unsafe { self.m_device.end_command_buffer(command_buffer)? };
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers);
        let graphics_queue = self.m_data.m_graphics_queue;
        unsafe {
            self.m_device.queue_submit(graphics_queue, &[submit_info], vk::Fence::null())?;
            self.m_device.queue_wait_idle(graphics_queue)?;
            self.m_device.free_command_buffers(self.m_data.m_command_pool, &command_buffers);
        }
        Ok(())
    }

    pub fn prepare_before_pass(&mut self, pass_update_after_recreate_swapchain: &dyn Fn(&VulkanRHI)) -> Result<bool> {
        let in_flight_fence = self.m_data.m_is_frame_in_flight_fences[self.m_current_frame_index];

        let result = unsafe{
            self.m_device.acquire_next_image_khr(
                self.m_data.m_swapchain,
                u64::MAX,
                self.m_data.m_image_available_for_render_semaphores[self.m_current_frame_index],
                vk::Fence::null(),
            )
        };

        let window = self.m_data.m_window.upgrade().unwrap();
        let window = window.as_ref();

        self.m_data.m_current_swapchain_image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => { 
                self.recreate_swapchain(window)?;
                pass_update_after_recreate_swapchain(&self);
                return Ok(true)
            },
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.m_data.m_images_in_flight[self.m_data.m_current_swapchain_image_index];
        if !image_in_flight.is_null() {
            unsafe { self.m_device.wait_for_fences(&[image_in_flight], true, u64::MAX) }?;
        }
        self.m_data.m_images_in_flight[self.m_data.m_current_swapchain_image_index] = in_flight_fence;

        let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe{
            self.m_device.begin_command_buffer(self.m_data.m_current_command_buffer, &info)?;
        }

        Ok(false)
    }

    pub fn submit_rendering(&mut self, pass_update_after_recreate_swapchain: &dyn Fn(&VulkanRHI)) -> Result<()> {
        let command_buffer = self.m_data.m_current_command_buffer;
        unsafe{ 
            self.m_device.end_command_buffer(command_buffer)?;
        } 
        
        let wait_semaphores = &[self.m_data.m_image_available_for_render_semaphores[self.m_current_frame_index]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[command_buffer];
        let signal_semaphores = &[self.m_data.m_image_finished_for_presentation_semaphores[self.m_current_frame_index]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        let in_flight_fence = self.m_data.m_is_frame_in_flight_fences[self.m_current_frame_index];
        unsafe { self.m_device.reset_fences(&[in_flight_fence]) }?;

        unsafe { self.m_device
            .queue_submit(self.m_data.m_graphics_queue, &[submit_info], in_flight_fence) }?;

        let swapchains = &[self.m_data.m_swapchain];
        let image_indices = &[self.m_data.m_current_swapchain_image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = unsafe { self.m_device.queue_present_khr(self.m_data.m_present_queue, &present_info) };
        
        let window = self.m_data.m_window.upgrade().unwrap();
        let window = window.as_ref();

        match result {
            Ok(vk::SuccessCode::SUBOPTIMAL_KHR) | Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                pass_update_after_recreate_swapchain(&self);
            }
            Err(e) => {
                return Err(anyhow!(e));
            }
            _=>{}
        }

        self.m_current_frame_index = (self.m_current_frame_index + 1) % K_MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
    
    pub fn push_event(&self, command_buffer: vk::CommandBuffer, event_name: &str, color: [f32; 4]) {
        if self.m_data.m_enable_debug_utils_label {
            let label_info = vk::DebugUtilsLabelEXT::builder()
                .label_name(event_name.as_bytes())
                .color(color);
            unsafe{
                self.m_instance.cmd_begin_debug_utils_label_ext(command_buffer, &label_info);
            }
        }
    }

    pub fn pop_event(&self, command_buffer: vk::CommandBuffer) {
        if self.m_data.m_enable_debug_utils_label {
            unsafe{
                self.m_instance.cmd_end_debug_utils_label_ext(command_buffer);
            }
        }
    }
    
    pub fn destroy_descriptor_set_layout(&self, layout: vk::DescriptorSetLayout) {
        unsafe{
            self.m_device.destroy_descriptor_set_layout(layout, None);
        }
    }
    
    pub fn destroy_shader_module(&self, shader: vk::ShaderModule) {
        unsafe{
            self.m_device.destroy_shader_module(shader, None);
        }
    }
    
    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.m_device.destroy_image_view(image_view, None);
        }
    }

    pub fn destroy_image(&self, image: vk::Image) {
        unsafe {
            self.m_device.destroy_image(image, None);
        }
    }

    pub fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        unsafe {
            self.m_device.destroy_framebuffer(framebuffer, None);
        }
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.m_device.destroy_pipeline(pipeline, None);
        }
    }

    pub fn destroy_pipeline_layout(&self, layout: vk::PipelineLayout) {
        unsafe {
            self.m_device.destroy_pipeline_layout(layout, None);
        }
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe {
            self.m_device.destroy_render_pass(render_pass, None);
        }
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.m_device.destroy_buffer(buffer, None);
        }
    }

    pub fn destroy_sampler(&self, sampler: vk::Sampler) {
        unsafe {
            self.m_device.destroy_sampler(sampler, None);
        }
    }

    pub fn free_memory(&self, memory:vk::DeviceMemory) {
        unsafe {
            self.m_device.free_memory(memory, None);
        }
    }

    pub fn map_memory(&self, memory: vk::DeviceMemory, offset: vk::DeviceSize, size: vk::DeviceSize, flags: vk::MemoryMapFlags) -> Result<*mut std::ffi::c_void> {
        Ok(unsafe{self.m_device.map_memory(
            memory, 
            offset, 
            size, 
            flags, 
        )?})
    }

    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe{self.m_device.unmap_memory(memory)}
    }
}

fn check_validation_layer_support(entry: &Entry, data: &VulkanRHIData) -> Result<bool> {
    let available_layers = unsafe {
        entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>()
    }; 

    let res = data.m_validation_layers.iter().all(|&layer_name|
        available_layers.contains(&layer_name)
    );

    Ok(res)
}

fn get_required_extensions(entry: &Entry, data: &VulkanRHIData) -> Result<Vec<*const i8>> {

    let binding = data.m_window.upgrade();
    let window = binding.as_ref().unwrap();

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if data.m_enable_validation_layers || data.m_enable_debug_utils_label {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");
        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
    }

    Ok(extensions)
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,   
) -> vk::Bool32 {
    let data = unsafe {*data};
    let message = unsafe { CStr::from_ptr(data.message)}.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    }else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    }else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    }else{
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

fn populate_debug_messenger_info(info: vk::DebugUtilsMessengerCreateInfoEXTBuilder) -> vk::DebugUtilsMessengerCreateInfoEXTBuilder{
    info
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        )
        .user_callback(Some(debug_callback))
}

fn create_instance(_window: &Window, entry: &Entry, data: &mut VulkanRHIData) -> Result<Instance> {
    if data.m_enable_validation_layers && !check_validation_layer_support(entry, data)? {
        error!("validation layers requested, but not available!");
    }

    data.m_vulkan_api_version = vk::make_version(1, 0, 0);

    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Example\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(data.m_vulkan_api_version);

    let extensions = get_required_extensions(entry, data)?;

    let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

    let validation_layers = data.m_validation_layers.iter().map(|&layer| layer.as_ptr()).collect::<Vec<_>>();

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&validation_layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder();
    debug_info = populate_debug_messenger_info(debug_info);

    if data.m_enable_validation_layers {
        info = info.push_next(&mut debug_info);
    }

    let instance = unsafe {
        entry.create_instance(&info, None)?
    };

    Ok(instance)
}

fn initialize_debug_messenger(instance: &Instance, data: &mut VulkanRHIData) -> Result<()> {
    if data.m_enable_validation_layers {
        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder();
        debug_info = populate_debug_messenger_info(debug_info);
        data.m_debug_messenger = unsafe {
            instance.create_debug_utils_messenger_ext(&debug_info, None)?
        };
    }
    Ok(())
}

fn create_window_surface(instance: &Instance, window: &Window, data: &mut VulkanRHIData) -> Result<()> {
    unsafe {
        data.m_surface = vk_window::create_surface(instance, window, window)?;
    }
    Ok(())
}

fn find_queue_families(instance: &Instance, data: &VulkanRHIData, physical_device: vk::PhysicalDevice) -> Result<QueueFamilyIndices> {
    let properties = unsafe {
        instance.get_physical_device_queue_family_properties(physical_device)
    };
    let mut indices = QueueFamilyIndices::default();
    for (index,properties) in properties.iter().enumerate() {
        if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) { 
            indices.graphics_family = Some(index as u32);
        }
        if unsafe {instance.get_physical_device_surface_support_khr(physical_device, index as u32, data.m_surface)?} {
            indices.present_family = Some(index as u32);
        }
        if indices.is_complete() {
            break;
        }
    }
    Ok(indices)
}

fn check_device_extension_support(instance: &Instance, data: &VulkanRHIData, physical_device: vk::PhysicalDevice) -> Result<bool> {
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device, None)?
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>()
    };

    Ok(data.m_device_extensions.iter().all(|e| extensions.contains(e)))
}
    
fn query_swapchain_support(instance: &Instance, data: &VulkanRHIData, physical_device: vk::PhysicalDevice) -> Result<SwapChainSupportDetails> { 
        unsafe {
            Ok(SwapChainSupportDetails { 
                capabilities: instance.get_physical_device_surface_capabilities_khr(physical_device, data.m_surface)?,
                formats: instance.get_physical_device_surface_formats_khr(physical_device, data.m_surface)?,
                present_modes: instance.get_physical_device_surface_present_modes_khr(physical_device, data.m_surface)?,
            })
        }
    }

fn is_device_suitable(instance: &Instance, data: &VulkanRHIData, physical_device: vk::PhysicalDevice) -> Result<bool> { 
    let queue_indices = find_queue_families(instance, data, physical_device)?;
    if !queue_indices.is_complete() {
        return Ok(false);
    }
    if !check_device_extension_support(instance, data, physical_device)? {
        return Ok(false);
    }
    let swapchain_support_details = query_swapchain_support(instance, data, physical_device)?;
    let is_swapchain_adequate =
        !swapchain_support_details.formats.is_empty() && !swapchain_support_details.present_modes.is_empty();

    if !is_swapchain_adequate {
        return Ok(false);
    }

    let features = unsafe{ instance.get_physical_device_features(physical_device) };
    if features.sampler_anisotropy != vk::TRUE {
        return Ok(false);
    }

    Ok(true)
}

fn initial_physical_device(instance: &Instance, data: &mut VulkanRHIData) -> Result<()>{
    let mut ranked_physical_devices = vec![];
    unsafe {
        for physical_device in instance.enumerate_physical_devices()?{
            let properties = instance.get_physical_device_properties(physical_device);
            let mut score = 0;
            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score = 1000;
            }
            else if properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                score = 100;
            }
            ranked_physical_devices.push((physical_device, score));
        }
    }
    ranked_physical_devices.sort_by(|a, b| b.1.cmp(&a.1));

    for (physical_device, _) in ranked_physical_devices {
        if (is_device_suitable(instance, data, physical_device))? {
            data.m_physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

fn find_supported_format(instance: &Instance, data: &VulkanRHIData, candidates: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> Result<vk::Format> {
    for &format in candidates {
        let properties = unsafe {
            instance.get_physical_device_format_properties(data.m_physical_device, format)
        };
        if tiling == vk::ImageTiling::LINEAR && properties.linear_tiling_features.contains(features) {
            return Ok(format);
        } else if tiling == vk::ImageTiling::OPTIMAL && properties.optimal_tiling_features.contains(features) {
            return Ok(format);
        }
    }
    Err(anyhow!("Failed to find supported format."))
}

fn create_logical_device(entry: &Entry, instance: &Instance, data: &mut VulkanRHIData) -> Result<Device> {

    data.m_queue_indices = find_queue_families(instance,data, data.m_physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(data.m_queue_indices.graphics_family.unwrap());
    unique_indices.insert(data.m_queue_indices.present_family.unwrap());

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices.
        iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = data.m_validation_layers.iter().map(|layer| layer.as_ptr()).collect::<Vec<_>>();
    let mut extensions = data.m_device_extensions.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .fragment_stores_and_atomics(true)
        .independent_blend(true)
        .sample_rate_shading(true);

    let device_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = unsafe {
        instance.create_device(data.m_physical_device, &device_info, None)?
    };

    unsafe {
        let vk_graphics_queue = device.get_device_queue(data.m_queue_indices.graphics_family.unwrap(), 0);
        data.m_graphics_queue = vk_graphics_queue;
        data.m_present_queue = device.get_device_queue(data.m_queue_indices.present_family.unwrap(), 0);
    }

    Ok(device)
}

fn create_command_pool(device: &Device, data: &mut VulkanRHIData) -> Result<()> {
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(data.m_queue_indices.graphics_family.unwrap());

    data.m_command_pool = unsafe { device.create_command_pool(&info, None)? };

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(data.m_queue_indices.graphics_family.unwrap());

    for i in 0..K_MAX_FRAMES_IN_FLIGHT as usize {
        data.m_command_pools[i] = unsafe { device.create_command_pool(&info, None)? };
    }

    Ok(())
}

fn create_command_buffers(device: &Device, data: &mut VulkanRHIData) -> Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    for i in 0..K_MAX_FRAMES_IN_FLIGHT{
        let allocate_info = allocate_info.command_pool(data.m_command_pools[i]);
        data.m_command_buffers[i] = unsafe { device.allocate_command_buffers(&allocate_info)?[0] };
    }

    Ok(())
}

use linkme::distributed_slice;

#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_UNIFORM_BUFFER: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_COMBINED_IMAGE_SAMPLER: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_INPUT_ATTACHMENT: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_UNIFORM_BUFFER_DYNAMIC: [u32];
#[distributed_slice]
pub static VULKAN_RHI_DESCRIPTOR_STORAGE_IMAGE: [u32];

fn create_descriptor_pool(device: &Device, data: &mut VulkanRHIData) -> Result<()> { 
    let pool_sizes = [
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER_DYNAMIC.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_STORAGE_BUFFER.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_UNIFORM_BUFFER.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_COMBINED_IMAGE_SAMPLER.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::INPUT_ATTACHMENT)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_INPUT_ATTACHMENT.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_UNIFORM_BUFFER_DYNAMIC.iter().sum()),
        vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(VULKAN_RHI_DESCRIPTOR_STORAGE_IMAGE.iter().sum()),
    ].into_iter().filter(|poolsize| poolsize.descriptor_count > 0).collect::<Vec<_>>();
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(1 + 1 + 1 + data.m_max_material_count + data.m_max_vertex_blending_mesh_count + 1 + 1);

    unsafe {
        data.m_descriptor_pool = device.create_descriptor_pool(&info, None)?;
    }
    Ok(())
}

fn create_sync_objects(device: &Device, data: &mut VulkanRHIData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    for i in 0..K_MAX_FRAMES_IN_FLIGHT {
        unsafe{
            data.m_image_available_for_render_semaphores[i] = device.create_semaphore(&semaphore_info, None)?;
            data.m_image_finished_for_presentation_semaphores[i] = device.create_semaphore(&semaphore_info, None)?;
            data.m_is_frame_in_flight_fences[i] = device.create_fence(&fence_info, None)?;
        }
        data.m_images_in_flight.push(vk::Fence::null());
    }

    Ok(())
}

fn choose_swapchain_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    available_formats
    .iter()
    .cloned()
    .find(|f| {
        f.format == vk::Format::B8G8R8A8_SRGB
        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
    })
    .unwrap_or_else(|| available_formats[0])
}

fn choose_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|&m| m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

fn choose_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D::builder()
            .width(window.inner_size().width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(window.inner_size().height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
            .build()
    }
}

fn create_swapchain(window: &Window, instance: &Instance, device: &Device, data: &mut VulkanRHIData) -> Result<()> {
    let swapchain_support_details = query_swapchain_support(instance, data, data.m_physical_device)?;

    let chosen_surface_format = choose_swapchain_surface_format(&swapchain_support_details.formats);
    let chosen_present_mode = choose_swapchain_present_mode(&swapchain_support_details.present_modes);
    let chosen_extent = choose_swapchain_extent(window, swapchain_support_details.capabilities);

    let mut image_count = swapchain_support_details.capabilities.min_image_count + 1;
    if swapchain_support_details.capabilities.max_image_count != 0 
        && image_count > swapchain_support_details.capabilities.max_image_count 
    {
        image_count = swapchain_support_details.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if data.m_queue_indices.graphics_family.unwrap() != data.m_queue_indices.present_family.unwrap() {
        queue_family_indices.push(data.m_queue_indices.graphics_family.unwrap());
        queue_family_indices.push(data.m_queue_indices.present_family.unwrap());
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(data.m_surface)
        
        .min_image_count(image_count)
        .image_format(chosen_surface_format.format)
        .image_color_space(chosen_surface_format.color_space)
        .image_extent(chosen_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(swapchain_support_details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(chosen_present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    unsafe {
        data.m_swapchain = device.create_swapchain_khr(&info, None)?;
        data.m_swapchain_images = device.get_swapchain_images_khr(data.m_swapchain)?;
        data.m_swapchain_image_format = chosen_surface_format.format;
        data.m_swapchain_extent = chosen_extent;
    }

    Ok(())
}

fn create_swapchain_image_views(device: &Device, data: &mut VulkanRHIData) -> Result<()> {
        data.m_swapchain_image_views = data.m_swapchain_images
            .iter()
            .map(|i| {
                create_image_view(
                    device,
                    *i, 
                    data.m_swapchain_image_format,
                    vk::ImageAspectFlags::COLOR,
                    vk::ImageViewType::_2D,
                    1,
                    1,
                ).unwrap()
            })
            .collect::<Vec<_>>();
        Ok(())
}

fn find_depth_format(instance: &Instance, data: &VulkanRHIData) -> Result<vk::Format> {
    find_supported_format(
        instance, data,
        &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT], 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}

fn create_depth_objects(instance: &Instance, device: &Device, data: &mut VulkanRHIData) -> Result<()> {
    data.m_depth_image_format = find_depth_format(instance, data)?;
    (data.m_depth_image, data.m_depth_image_memory) = vulkan_util::create_image(
        instance,
        device,
        data.m_physical_device,
        data.m_swapchain_extent.width,
        data.m_swapchain_extent.height,

        data.m_depth_image_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageCreateFlags::empty(),
        1,
        1,
    )?;

        data.m_depth_image_view = vulkan_util::create_image_view(
            device,
            data.m_depth_image,
            data.m_depth_image_format,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageViewType::_2D,
            1,
            1,
        )?;

        Ok(())
    }