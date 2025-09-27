use std::{array, collections::{HashMap, HashSet}, ffi::CStr, mem::transmute, os::raw::c_void, sync::Weak};
use anyhow::{anyhow, Result};
use log::*;
use thiserror::Error;
use vulkanalia::{loader::{LibloadingLoader, LIBRARY}, prelude::v1_0::*, vk::{BufferUsageFlags, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension, MemoryPropertyFlags}, window as vk_window, Version};
use winit::window::Window;
use crate::{runtime::function::render::{interface::{rhi::{RHIInitInfo, RHI}, rhi_struct::{QueueFamilyIndices, RHIBuffer, RHIClearValue, RHICommandBuffer, RHICommandPool, RHIDepthImageDesc, RHIDescriptorPool, RHIDescriptorSet, RHIDescriptorSetAllocateInfo, RHIDescriptorSetLayout, RHIDescriptorSetLayoutCreateInfo, RHIDeviceMemory, RHIExtent2D, RHIFence, RHIFramebuffer, RHIFramebufferCreateInfo, RHIGraphicsPipelineCreateInfo, RHIImage, RHIImageView, RHIOffset2D, RHIPipeline, RHIPipelineLayout, RHIPipelineLayoutCreateInfo, RHIQueue, RHIRect2D, RHIRenderPass, RHIRenderPassBeginInfo, RHIRenderPassCreateInfo, RHISampler, RHISemaphore, RHIShader, RHISwapChainDesc, RHIViewport, RHIWriteDescriptorSet, SwapChainSupportDetails}, vulkan::{vulkan_rhi_resource::{VulkanBuffer, VulkanBufferView, VulkanCommandBuffer, VulkanCommandPool, VulkanDescriptorPool, VulkanDescriptorSet, VulkanDescriptorSetLayout, VulkanDeviceMemory, VulkanFence, VulkanFramebuffer, VulkanImage, VulkanImageView, VulkanPipeline, VulkanPipelineLayout, VulkanQueue, VulkanRenderPass, VulkanSampler, VulkanSemaphore, VulkanShader}, vulkan_util::VulkanUtil}}, render_type::{RHIBufferUsageFlags, RHIDefaultSamplerType, RHIDeviceSize, RHIFormat, RHIMemoryMapFlags, RHIMemoryPropertyFlags, RHIPipelineBindPoint, RHISubpassContents}}};

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub struct SuitabilityError(pub &'static str);

#[derive(Default)]
pub struct VulkanRHI {

    m_graphics_queue: Option<Box<dyn RHIQueue>>,

    m_swapchain_image_format: RHIFormat,
    m_swapchain_image_views: Vec<Box<dyn RHIImageView>>,
    m_swapchain_extent: RHIExtent2D,
    m_viewport: RHIViewport,
    m_scissor: RHIRect2D,

    m_depth_image_format: RHIFormat,
    m_depth_image_view: Option<Box<dyn RHIImageView>>,

    m_rhi_is_frame_in_flight_fences: Option<[Box<dyn RHIFence>; Self::K_MAX_FRAMES_IN_FLIGHT as usize]>,

    m_descriptor_pool : Option<Box<dyn RHIDescriptorPool>>,

    m_rhi_command_pool: Option<Box<dyn RHICommandPool>>,

    m_command_buffers: Option<[Box<dyn RHICommandBuffer>; Self::K_MAX_FRAMES_IN_FLIGHT as usize]>,
    m_current_command_buffer: Option<Box<dyn RHICommandBuffer>>,

    m_queue_indices: QueueFamilyIndices,

    m_window: Weak<Window>,
    m_entry: Option<Entry>,
    pub m_instance: Option<Instance>,
    m_surface: vk::SurfaceKHR,
    pub m_physical_device: vk::PhysicalDevice,
    pub m_device: Option<Device>,
    m_present_queue: vk::Queue,

    m_swapchain: vk::SwapchainKHR,
    m_swapchain_images: Vec<vk::Image>,

    m_depth_image: Option<Box<dyn RHIImage>>,
    m_depth_image_memory: vk::DeviceMemory,

    m_swapchain_framebuffers: Vec<vk::Framebuffer>,

    // m_assert_allocator: Option<gpu_allocator::vulkan::Allocator>,

    m_vk_descriptor_pool: vk::DescriptorPool,

    m_current_frame_index: u8,
    m_command_pools: [vk::CommandPool; Self::K_MAX_FRAMES_IN_FLIGHT as usize],
    m_vk_command_buffers: [vk::CommandBuffer; Self::K_MAX_FRAMES_IN_FLIGHT as usize],
    m_image_available_for_render_semaphores: [vk::Semaphore; Self::K_MAX_FRAMES_IN_FLIGHT as usize],
    m_image_finished_for_presentation_semaphores: [vk::Semaphore; Self::K_MAX_FRAMES_IN_FLIGHT as usize],
    m_image_available_for_texturescopy_semaphores: Option<[Box<dyn RHISemaphore>; Self::K_MAX_FRAMES_IN_FLIGHT as usize]>,
    m_is_frame_in_flight_fences: [vk::Fence; Self::K_MAX_FRAMES_IN_FLIGHT as usize],

    m_vk_current_command_buffer: vk::CommandBuffer,

    m_current_swapchain_image_index: u32,

    m_validation_layers: Vec<vk::ExtensionName>,
    m_vulkan_api_version: u32,

    m_device_extensions: Vec<vk::ExtensionName>,

    m_linear_sampler: Option<Box<dyn RHISampler>>,
    m_nearest_sampler: Option<Box<dyn RHISampler>>,
    m_mipmap_sampler_map: HashMap<u32, Box<dyn RHISampler>>,

    m_enable_validation_layers: bool,
    m_enable_debug_utils_label: bool,

    m_max_vertex_blending_mesh_count: u32,
    m_max_material_count: u32,

    m_debug_messenger: vk::DebugUtilsMessengerEXT,    
}

unsafe impl Send for VulkanRHI {}
unsafe impl Sync for VulkanRHI {}

impl RHI for VulkanRHI {
    fn initialize(&mut self, info: RHIInitInfo) -> Result<()> {

        self.m_max_material_count = 256; 
        self.m_max_vertex_blending_mesh_count = 256;

        let window_system = info.window_system;
        self.m_window = window_system.get_window();
        let window_size = window_system.get_window_size();

        self.m_viewport = RHIViewport {
            x: 0.0,
            y: 0.0,
            width: window_size.0 as f32,
            height: window_size.1 as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        self.m_scissor = RHIRect2D {
            offset: RHIOffset2D { x: 0, y: 0 },
            extent: RHIExtent2D {
                width: window_size.0,
                height: window_size.1,
            },
        };

        self.m_entry  = Some(unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            Entry::new(loader).map_err(|e| anyhow!("{}", e))?
        });

        if cfg!(debug_assertions) {
            self.m_validation_layers = vec![
                vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation"),
            ];
            self.m_enable_validation_layers = true;
            self.m_enable_debug_utils_label = true;
        }
        else{
            self.m_enable_validation_layers = false;
            self.m_enable_debug_utils_label = false;
        }

        self.create_instance()?;
        self.initialize_debug_messenger()?;
        self.create_window_surface()?;
        self.initial_physical_device()?;
        self.create_logical_device()?;
        self.create_command_pool()?;
        self.create_command_buffers()?;
        self.create_descriptor_pool()?;
        self.create_sync_primitives()?;
        self.create_swapchain()?;
        self.create_swapchain_image_views()?;
        self.create_framebuffer_image_and_views()?;
        self.create_assert_allocator()?;

        Ok(())
    }

    fn allocate_descriptor_sets(&self, allocate_info:&RHIDescriptorSetAllocateInfo) -> Result<Vec<Box<dyn RHIDescriptorSet>>> {
        let layouts = allocate_info.set_layouts.iter().map(|layout| {
            layout.as_any().downcast_ref::<VulkanDescriptorSetLayout>().unwrap().get_resource()
        }).collect::<Vec<_>>();
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(allocate_info.descriptor_pool.as_any().downcast_ref::<VulkanDescriptorPool>().unwrap().get_resource())
            .set_layouts(&layouts)
            .build();
        let descriptor_sets = unsafe{
            self.m_device.as_ref().unwrap().allocate_descriptor_sets(&allocate_info)?
        };
        let descriptor_sets = descriptor_sets.iter().map(|descriptor_set|{
            Box::new(VulkanDescriptorSet::new(descriptor_set.clone())) as Box<dyn RHIDescriptorSet>
        }).collect::<Vec<_>>();
        Ok(descriptor_sets)
    }

    fn create_swapchain(&mut self) -> Result<()> {
        let swapchain_support_details = self.query_swapchain_support(self.m_physical_device)?;

        let chosen_surface_format = Self::choose_swapchain_surface_format(&swapchain_support_details.formats);
        let chosen_present_mode = Self::choose_swapchain_present_mode(&swapchain_support_details.present_modes);
        let window = self.m_window.upgrade().unwrap();
        let chosen_extent = Self::choose_swapchain_extent(&window, swapchain_support_details.capabilities);

        let mut image_count = swapchain_support_details.capabilities.min_image_count + 1;
        if swapchain_support_details.capabilities.max_image_count != 0 
            && image_count > swapchain_support_details.capabilities.max_image_count 
        {
            image_count = swapchain_support_details.capabilities.max_image_count;
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if self.m_queue_indices.graphics_family.unwrap() != self.m_queue_indices.present_family.unwrap() {
            queue_family_indices.push(self.m_queue_indices.graphics_family.unwrap());
            queue_family_indices.push(self.m_queue_indices.present_family.unwrap());
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };
    
        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.m_surface)
            
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
            self.m_swapchain = self.m_device.as_ref().unwrap().create_swapchain_khr(&info, None)?;
            self.m_swapchain_images = self.m_device.as_ref().unwrap().get_swapchain_images_khr(self.m_swapchain)?;
            self.m_swapchain_image_format = RHIFormat::from_raw(chosen_surface_format.format.as_raw());
            self.m_swapchain_extent = RHIExtent2D {
                width: chosen_extent.width,
                height: chosen_extent.height,
            };
        }

        Ok(())
    }

    fn create_swapchain_image_views(&mut self) -> Result<()> {
        let format = vk::Format::from_raw(self.m_swapchain_image_format.as_raw());
        self.m_swapchain_image_views = self
            .m_swapchain_images
            .iter()
            .map(|i| {
                let image_view = VulkanUtil::create_image_view(
                    self.m_device.as_ref().unwrap(), 
                    *i, 
                    format,
                    vk::ImageAspectFlags::COLOR,
                    vk::ImageViewType::_2D,
                    1,
                    1,
                ).unwrap();
                Box::new(VulkanImageView::new(image_view)) as Box<dyn RHIImageView>
            })
            .collect::<Vec<_>>();
        Ok(())
    }

    fn get_or_create_default_sampler(&mut self, sampler_type: RHIDefaultSamplerType) -> Result<&Box<dyn RHISampler>> {
        match sampler_type {
            RHIDefaultSamplerType::Linear => {
                if self.m_linear_sampler.is_some() {
                    return Ok(&self.m_linear_sampler.as_ref().unwrap());
                }
                let properties = unsafe{self.m_instance.as_ref().unwrap().get_physical_device_properties(self.m_physical_device)};
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

                let sampler = unsafe {
                    self.m_device.as_ref().unwrap().create_sampler(&create_info, None)?
                };
                self.m_linear_sampler = Some(Box::new(VulkanSampler::new(sampler)) as Box<dyn RHISampler>);
                return Ok(&self.m_linear_sampler.as_ref().unwrap());
            }
            RHIDefaultSamplerType::Nearest => {
                if self.m_nearest_sampler.is_some() {
                    return Ok(&self.m_nearest_sampler.as_ref().unwrap());
                }
                let properties = unsafe{self.m_instance.as_ref().unwrap().get_physical_device_properties(self.m_physical_device)};
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

                let sampler = unsafe {
                    self.m_device.as_ref().unwrap().create_sampler(&create_info, None)?
                };
                self.m_linear_sampler = Some(Box::new(VulkanSampler::new(sampler)) as Box<dyn RHISampler>);
                return Ok(&self.m_linear_sampler.as_ref().unwrap());
            }
        }
    }

    fn create_shader_module(&self, data: &[u8]) -> Result<Box<dyn RHIShader>> {
        let shader_module = VulkanUtil::create_shader_module(self.m_device.as_ref().unwrap(), data)?;
        Ok(Box::new(VulkanShader::new(shader_module)) as Box<dyn RHIShader>)
    }

    fn create_buffer(&self,size: RHIDeviceSize, usage: RHIBufferUsageFlags, properties: RHIMemoryPropertyFlags) 
        -> Result<(Box<dyn RHIBuffer>, Box<dyn RHIDeviceMemory>)> 
    {
        let (buffer,memory) = VulkanUtil::create_buffer(
            &self.m_instance.as_ref().unwrap(), 
            &self.m_device.as_ref().unwrap(), 
            self.m_physical_device, size, 
            unsafe{BufferUsageFlags::from_bits_unchecked(usage.bits())},
            unsafe{MemoryPropertyFlags::from_bits_unchecked(properties.bits())},
        )?;
        let buffer = Box::new(VulkanBuffer::new(buffer)) as Box<dyn RHIBuffer>;
        let memory = Box::new(VulkanDeviceMemory::new(memory)) as Box<dyn RHIDeviceMemory>;
        Ok((buffer, memory))
    }

    fn copy_buffer(&self, src: &Box<dyn RHIBuffer>, dst: &Box<dyn RHIBuffer>, src_offset: RHIDeviceSize, dst_offset: RHIDeviceSize, size: RHIDeviceSize) -> Result<()> {
        let src_buffer = src.as_any().downcast_ref::<VulkanBuffer>().unwrap().get_resource();
        let dst_buffer = dst.as_any().downcast_ref::<VulkanBuffer>().unwrap().get_resource();

        VulkanUtil::copy_buffer(
            self,
            &self.m_device.as_ref().unwrap(),
            src_buffer,
            dst_buffer,
            src_offset,
            dst_offset,
            size,
        )?;

        Ok(())
    }

    fn create_texture_image(&self, width: u32, height: u32, pixels: &[u8], format: RHIFormat, mip_levels: u32) -> Result<(Box<dyn RHIImage>, Box<dyn RHIDeviceMemory>, Box<dyn RHIImageView>)> {
        let (image, memory, image_view) = VulkanUtil::create_texture_image(
            self,
            width,
            height,
            pixels,
            vk::Format::from_raw(format.as_raw()),
            mip_levels,
        )?;
        Ok((
            Box::new(VulkanImage::new(image)), 
            Box::new(VulkanDeviceMemory::new(memory)),
            Box::new(VulkanImageView::new(image_view)),
        ))
    }

    fn create_descriptor_set_layout(&self, create_info:&RHIDescriptorSetLayoutCreateInfo) -> Result<Box<dyn RHIDescriptorSetLayout>>{
        let bindings = create_info.bindings.iter().map(|binding| {
            vk::DescriptorSetLayoutBinding::builder()
                .binding(binding.binding)
                .descriptor_type(vk::DescriptorType::from_raw(binding.descriptor_type.as_raw()))
                .descriptor_count(binding.descriptor_count)
                .stage_flags(unsafe {
                    vk::ShaderStageFlags::from_bits_unchecked(binding.stage_flags.bits())
                })
                .build()
        }).collect::<Vec<_>>();


        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .flags(unsafe{
                vk::DescriptorSetLayoutCreateFlags::from_bits_unchecked(create_info.flags.bits())
            });

        let descriptor_set_layout = unsafe {
            self.m_device.as_ref().unwrap().create_descriptor_set_layout(&info, None)?
        };
        Ok(Box::new(VulkanDescriptorSetLayout::new(descriptor_set_layout)))
    }

    fn create_framebuffer(&self, create_info: &RHIFramebufferCreateInfo) -> Result<Box<dyn RHIFramebuffer>> {
        let attachments = create_info.attachments.iter().map(|attachment| {
            attachment.as_any().downcast_ref::<VulkanImageView>().unwrap().get_resource()
        }).collect::<Vec<_>>();

        let info = vk::FramebufferCreateInfo::builder()
            .flags(unsafe{vk::FramebufferCreateFlags::from_bits_unchecked(create_info.flags.bits())})
            .render_pass(create_info.render_pass.as_any().downcast_ref::<VulkanRenderPass>().unwrap().get_resource())
            .attachments(&attachments)
            .width(create_info.width)
            .height(create_info.height)
            .layers(create_info.layers)
            .build();

        let framebuffer = unsafe {
            self.m_device.as_ref().unwrap().create_framebuffer(&info, None)?
        };
        Ok(Box::new(VulkanFramebuffer::new(framebuffer)))
    }
    
    fn create_graphics_pipelines(&self, create_info:&[RHIGraphicsPipelineCreateInfo]) -> Result<Vec<Box<dyn RHIPipeline>>> {

        let pipeline_create_infos = create_info.iter().map(|info|{
            let stages = info.stages.iter().map(|stage| {
                let shader_module = stage.module.as_any().downcast_ref::<VulkanShader>().unwrap().get_resource();
                vk::PipelineShaderStageCreateInfo::builder()
                    .flags(unsafe { vk::PipelineShaderStageCreateFlags::from_bits_unchecked(stage.flags.bits()) })
                    .stage(unsafe { vk::ShaderStageFlags::from_bits_unchecked(stage.stage.bits()) })
                    .module(shader_module)
                    .name(stage.name.as_bytes())
                    // .specialization_info()
                    .build()
            }).collect::<Vec<_>>();

            let vk_binding_descriptions = info.vertex_input_state.vertex_binding_descriptions.iter().map(|binding| {
                vk::VertexInputBindingDescription::builder()
                    .binding(binding.binding)
                    .stride(binding.stride)
                    .input_rate(vk::VertexInputRate::from_raw(binding.input_rate.as_raw()))
                    .build()
            }).collect::<Vec<_>>();

            let vk_attribute_descriptions = info.vertex_input_state.vertex_attribute_descriptions.iter().map(|attribute| {
                vk::VertexInputAttributeDescription::builder()
                    .location(attribute.location)
                    .binding(attribute.binding)
                    .format(vk::Format::from_raw(attribute.format.as_raw()))
                    .offset(attribute.offset)
                    .build()
            }).collect::<Vec<_>>();

            let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineVertexInputStateCreateFlags::from_bits_unchecked(info.vertex_input_state.flags.bits())})
                .vertex_binding_descriptions(&vk_binding_descriptions)
                .vertex_attribute_descriptions(&vk_attribute_descriptions);

            let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineInputAssemblyStateCreateFlags::from_bits_unchecked(info.input_assembly_state.flags.bits())})
                .topology(unsafe{vk::PrimitiveTopology::from_raw(info.input_assembly_state.topology.as_raw())})
                .primitive_restart_enable(info.input_assembly_state.primitive_restart_enable);

            //tessellation state

            let viewports = info.viewport_state.viewports.iter().map(|viewport|{
                vk::Viewport::builder()
                    .x(viewport.x)
                    .y(viewport.y)
                    .width(viewport.width)
                    .height(viewport.height)
                    .min_depth(viewport.min_depth)
                    .max_depth(viewport.max_depth)
                    .build()
            }).collect::<Vec<_>>();

            let scissors = info.viewport_state.scissors.iter().map(|scissor|{
                vk::Rect2D::builder()
                    .offset(vk::Offset2D { x: scissor.offset.x, y: scissor.offset.y })
                    .extent(vk::Extent2D { width: scissor.extent.width, height: scissor.extent.height })
                    .build()
            }).collect::<Vec<_>>();

            let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineViewportStateCreateFlags::from_bits_unchecked(info.viewport_state.flags.bits())})
                .viewports(&viewports)
                .scissors(&scissors);

            let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineRasterizationStateCreateFlags::from_bits_unchecked(info.rasterization_state.flags.bits())})
                .depth_clamp_enable(info.rasterization_state.depth_clamp_enable)
                .rasterizer_discard_enable(info.rasterization_state.rasterizer_discard_enable)
                .polygon_mode(vk::PolygonMode::from_raw(info.rasterization_state.polygon_mode.as_raw()))
                .cull_mode(unsafe{vk::CullModeFlags::from_bits_unchecked(info.rasterization_state.cull_mode.bits())})
                .front_face(vk::FrontFace::from_raw(info.rasterization_state.front_face.as_raw()))
                .depth_bias_enable(info.rasterization_state.depth_bias_enable)
                .depth_bias_constant_factor(info.rasterization_state.depth_bias_constant_factor)
                .depth_bias_clamp(info.rasterization_state.depth_bias_clamp)
                .depth_bias_slope_factor(info.rasterization_state.depth_bias_slope_factor)
                .line_width(info.rasterization_state.line_width);

            let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineMultisampleStateCreateFlags::from_bits_unchecked(info.multisample_state.flags.bits())})
                .rasterization_samples(unsafe{vk::SampleCountFlags::from_bits_unchecked(info.multisample_state.rasterization_samples.bits())})
                .sample_shading_enable(info.multisample_state.sample_shading_enable)
                .min_sample_shading(info.multisample_state.min_sample_shading)
                //.sample_mask()
                .alpha_to_coverage_enable(info.multisample_state.alpha_to_coverage_enable)
                .alpha_to_one_enable(info.multisample_state.alpha_to_one_enable);

            let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .min_depth_bounds(0.0)
                .max_depth_bounds(1.0)
                .stencil_test_enable(false);

            let attachements = info.color_blend_state.attachments.iter().map(|attachment| {
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(attachment.blend_enable)
                    .src_color_blend_factor(vk::BlendFactor::from_raw(attachment.src_color_blend_factor.as_raw()))
                    .dst_color_blend_factor(vk::BlendFactor::from_raw(attachment.dst_color_blend_factor.as_raw()))
                    .color_blend_op(vk::BlendOp::from_raw(attachment.color_blend_op.as_raw()))
                    .src_alpha_blend_factor(vk::BlendFactor::from_raw(attachment.src_alpha_blend_factor.as_raw()))
                    .dst_alpha_blend_factor(vk::BlendFactor::from_raw(attachment.dst_alpha_blend_factor.as_raw()))
                    .alpha_blend_op(vk::BlendOp::from_raw(attachment.alpha_blend_op.as_raw()))
                    .color_write_mask(unsafe{vk::ColorComponentFlags::from_bits_unchecked(attachment.color_write_mask.bits())})
            }).collect::<Vec<_>>();

            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .flags(unsafe{vk::PipelineColorBlendStateCreateFlags::from_bits_unchecked(info.color_blend_state.flags.bits())})
                .logic_op_enable(info.color_blend_state.logic_op_enable)
                .logic_op(vk::LogicOp::from_raw(info.color_blend_state.logic_op.as_raw()))
                .attachments(&attachements)
                .blend_constants(info.color_blend_state.blend_constants);

            let dynamic_state = match info.dynamic_state {
                Some(ref dynamic_state) => {
                    let dynamic_states = dynamic_state.dynamic_states.iter().map(|dynamic_state| {
                        vk::DynamicState::from_raw(dynamic_state.as_raw())
                    }).collect::<Vec<_>>();

                    vk::PipelineDynamicStateCreateInfo::builder()
                        .flags(unsafe {
                            vk::PipelineDynamicStateCreateFlags::from_bits_unchecked(dynamic_state.flags.bits())
                        })
                        .dynamic_states(&dynamic_states)
                        .build()
                }
                None => {
                    vk::PipelineDynamicStateCreateInfo::builder()
                    .build()
                }
            };

            vk::GraphicsPipelineCreateInfo::builder()
                .flags(unsafe{vk::PipelineCreateFlags::from_bits_unchecked(info.flags.bits())})
                .stages(&stages)
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                // .tessellation_state()
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                .depth_stencil_state(&depth_stencil_state)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state)
                .layout(info.layout.as_any().downcast_ref::<VulkanPipelineLayout>().unwrap().get_resource())
                .render_pass(info.render_pass.as_any().downcast_ref::<VulkanRenderPass>().unwrap().get_resource())
                .subpass(info.subpass)
                // .base_pipeline_handle()
                // .base_pipeline_index()
                .build()
        }).collect::<Vec<_>>();

        let pipelines = unsafe { self.m_device.as_ref().unwrap().create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None)?.0 };

        Ok(
           pipelines.iter().map(|pipeline|{
                Box::new(VulkanPipeline::new(*pipeline)) as Box<dyn RHIPipeline>
            }).collect::<Vec<_>>()
        )
    }

    fn create_pipeline_layout(&self, create_info: &RHIPipelineLayoutCreateInfo) -> Result<Box<dyn RHIPipelineLayout>> {
        let vk_set_layouts = create_info.set_layouts.iter().map(|layout| {
            layout.as_any().downcast_ref::<VulkanDescriptorSetLayout>().unwrap().get_resource()
        }).collect::<Vec<_>>();

        let vk_push_constant_ranges = create_info.push_constant_ranges.iter().map(|range| {
            vk::PushConstantRange::builder()
                .stage_flags(unsafe{vk::ShaderStageFlags::from_bits_unchecked(range.stage_flags.bits())})
                .offset(range.offset)
                .size(range.size)
                .build()
        }).collect::<Vec<_>>();

        let info = vk::PipelineLayoutCreateInfo::builder()
            .flags(unsafe{vk::PipelineLayoutCreateFlags::from_bits_unchecked(create_info.flags.bits())})
            .set_layouts(&vk_set_layouts)
            .push_constant_ranges(&vk_push_constant_ranges)
            .build();

        let pipeline_layout = unsafe {
            self.m_device.as_ref().unwrap().create_pipeline_layout(&info, None)?
        };
        Ok(Box::new(VulkanPipelineLayout::new(pipeline_layout)))
    }

    fn create_render_pass(&self, create_info: &RHIRenderPassCreateInfo) -> Result<Box<dyn RHIRenderPass>> {
        let vk_attachments = create_info.attachments.iter().map(|attachment| {
            vk::AttachmentDescription::builder()
                .format(vk::Format::from_raw(attachment.format.as_raw()))
                .samples(unsafe{vk::SampleCountFlags::from_bits_unchecked(attachment.samples.bits())})
                .load_op(vk::AttachmentLoadOp::from_raw(attachment.load_op.as_raw()))
                .store_op(vk::AttachmentStoreOp::from_raw(attachment.store_op.as_raw()))
                .stencil_load_op(vk::AttachmentLoadOp::from_raw(attachment.stencil_load_op.as_raw()))
                .stencil_store_op(vk::AttachmentStoreOp::from_raw(attachment.stencil_store_op.as_raw()))
                .initial_layout(vk::ImageLayout::from_raw(attachment.initial_layout.as_raw()))
                .final_layout(vk::ImageLayout::from_raw(attachment.final_layout.as_raw()))
                .build()
        }).collect::<Vec<_>>();

        let vk_subpasses = create_info.subpasses.iter().map(|subpass| {
            let depth_stencil_attachment = vk::AttachmentReference::builder()
                .attachment(subpass.depth_stencil_attachment.attachment)
                .layout(vk::ImageLayout::from_raw(subpass.depth_stencil_attachment.layout.as_raw()))
                .build();
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::from_raw(subpass.pipeline_bind_point.as_raw()))
                .input_attachments(&subpass.input_attachments.iter().map(|attachment| {
                    vk::AttachmentReference::builder()
                        .attachment(attachment.attachment)
                        .layout(vk::ImageLayout::from_raw(attachment.layout.as_raw()))
                }).collect::<Vec<_>>())
                .color_attachments(&subpass.color_attachments.iter().map(|attachment| {
                    vk::AttachmentReference::builder()
                        .attachment(attachment.attachment)
                        .layout(vk::ImageLayout::from_raw(attachment.layout.as_raw()))
                }).collect::<Vec<_>>())
                .resolve_attachments(&subpass.resolve_attachments.iter().map(|attachment| {
                    vk::AttachmentReference::builder()
                        .attachment(attachment.attachment)
                        .layout(vk::ImageLayout::from_raw(attachment.layout.as_raw()))
                }).collect::<Vec<_>>())
                .depth_stencil_attachment(&depth_stencil_attachment)
                .preserve_attachments(&subpass.preserve_attachments)
                .build()
        }).collect::<Vec<_>>();

        let vk_dependencies = create_info.dependencies.iter().map(|dependency| {
            vk::SubpassDependency::builder()
                .src_subpass(dependency.src_subpass)
                .dst_subpass(dependency.dst_subpass)
                .src_stage_mask(unsafe{vk::PipelineStageFlags::from_bits_unchecked(dependency.src_stage_mask.bits())})
                .dst_stage_mask(unsafe{vk::PipelineStageFlags::from_bits_unchecked(dependency.dst_stage_mask.bits())})
                .src_access_mask(unsafe{vk::AccessFlags::from_bits_unchecked(dependency.src_access_mask.bits())})
                .dst_access_mask(unsafe{vk::AccessFlags::from_bits_unchecked(dependency.dst_access_mask.bits())})
                .dependency_flags(unsafe{vk::DependencyFlags::from_bits_unchecked(dependency.dependency_flags.bits())})
                .build()
        }).collect::<Vec<_>>();

        let info = vk::RenderPassCreateInfo::builder()
            .flags(unsafe{vk::RenderPassCreateFlags::from_bits_unchecked(create_info.flags.bits())})
            .attachments(&vk_attachments)
            .subpasses(&vk_subpasses)
            .dependencies(&vk_dependencies)
            .build();

        let render_pass = unsafe {
            self.m_device.as_ref().unwrap().create_render_pass(&info, None)?
        };
        Ok(Box::new(VulkanRenderPass::new(render_pass)))
    }
    
    fn cmd_begin_render_pass(&self, command_buffer: &Box<dyn RHICommandBuffer>, render_pass_begin_info: &RHIRenderPassBeginInfo, contents: RHISubpassContents) {
        let offset_2d = vk::Offset2D {
            x: render_pass_begin_info.render_area.offset.x,
            y: render_pass_begin_info.render_area.offset.y,
        };
        let extent_2d = vk::Extent2D {
            width: render_pass_begin_info.render_area.extent.width,
            height: render_pass_begin_info.render_area.extent.height,
        };
        let rect_2d = vk::Rect2D {
            offset: offset_2d,
            extent: extent_2d,
        };

        let clear_values = render_pass_begin_info.clear_values.iter().map(|clear_value| {
            match clear_value {
                RHIClearValue::Color(value) => {
                    vk::ClearValue {
                        color: unsafe { transmute(*value) },
                    }
                },
                RHIClearValue::DepthStencil(value) => {
                    vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: value.depth,
                            stencil: value.stencil,
                        },
                    }
                }
            }
        }).collect::<Vec<_>>();

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass_begin_info.render_pass.as_any().downcast_ref::<VulkanRenderPass>().unwrap().get_resource())
            .framebuffer(render_pass_begin_info.framebuffer.as_any().downcast_ref::<VulkanFramebuffer>().unwrap().get_resource())
            .render_area(rect_2d)
            .clear_values(&clear_values)
            .build();

        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();

        unsafe{
            self.m_device.as_ref().unwrap().cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::from_raw(contents.as_raw()));
        }

    }

    fn cmd_end_render_pass(&self, command_buffer: &Box<dyn RHICommandBuffer>) {
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe{
            self.m_device.as_ref().unwrap().cmd_end_render_pass(command_buffer);
        }
    }

    fn cmd_bind_pipeline(&self, command_buffer: &Box<dyn RHICommandBuffer>, pipeline_bind_point: RHIPipelineBindPoint, pipeline: &Box<dyn RHIPipeline>) {
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        let pipeline = pipeline.as_any().downcast_ref::<VulkanPipeline>().unwrap().get_resource();
        unsafe{
            self.m_device.as_ref().unwrap().cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::from_raw(pipeline_bind_point.as_raw()), pipeline);
        }
    }
    
    fn cmd_set_viewport(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_viewport: u32, viewports: &[RHIViewport]) {
        let vk_viewports = viewports.iter().map(|viewport| {
            vk::Viewport::builder()
                .x(viewport.x)
                .y(viewport.y)
                .width(viewport.width)
                .height(viewport.height)
                .min_depth(viewport.min_depth)
                .max_depth(viewport.max_depth)
                .build()
        }).collect::<Vec<_>>();
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().cmd_set_viewport(command_buffer, first_viewport, &vk_viewports);
        }
    }

    fn cmd_set_scissor(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_scissor: u32, scissors: &[RHIRect2D]) {
        let vk_scissors = scissors.iter().map(|scissor| {
            vk::Rect2D::builder()
                .offset(vk::Offset2D {
                    x: scissor.offset.x as i32,
                    y: scissor.offset.y as i32,
                })
                .extent(vk::Extent2D {
                    width: scissor.extent.width as u32,
                    height: scissor.extent.height as u32,
                })
        }).collect::<Vec<_>>();
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().cmd_set_scissor(command_buffer, first_scissor, &vk_scissors);
        }
    }

    fn cmd_bind_vertex_buffers(&self, command_buffer: &Box<dyn RHICommandBuffer>, first_binding: u32, buffers: &[&Box<dyn RHIBuffer>], offsets: &[RHIDeviceSize]) {
        let vk_buffers = buffers.iter().map(|buffer| {
            buffer.as_any().downcast_ref::<VulkanBuffer>().unwrap().get_resource()
        }).collect::<Vec<_>>();
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().cmd_bind_vertex_buffers(command_buffer, first_binding, &vk_buffers, offsets);
        }
    }

    fn cmd_bind_descriptor_sets(&self, command_buffer: &Box<dyn RHICommandBuffer>, pipeline_bind_point: RHIPipelineBindPoint, layout: &Box<dyn RHIPipelineLayout>, first_set: u32, descriptor_sets: &[&Box<dyn RHIDescriptorSet>], dynamic_offsets: &[u32]) {
        let vk_descriptor_sets = descriptor_sets.iter().map(|set| {
            set.as_any().downcast_ref::<VulkanDescriptorSet>().unwrap().get_resource()
        }).collect::<Vec<_>>();
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        let layout = layout.as_any().downcast_ref::<VulkanPipelineLayout>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().cmd_bind_descriptor_sets(
                command_buffer, 
                vk::PipelineBindPoint::from_raw(pipeline_bind_point.as_raw()), 
                layout, 
                first_set, 
                &vk_descriptor_sets, 
                dynamic_offsets,
            );
        }
    }
    
    fn cmd_draw(&self, command_buffer: &Box<dyn RHICommandBuffer>, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32){
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().cmd_draw(
                command_buffer, 
                vertex_count, 
                instance_count, 
                first_vertex, 
                first_instance,
            );
        }
    }

    fn update_descriptor_sets(&self, writes: &[RHIWriteDescriptorSet]) -> Result<()> {
        let vk_writes = writes.iter().map(|write| {
            let image_info = write.image_info.iter().map(|info| {
                vk::DescriptorImageInfo::builder()
                    .sampler(info.sampler.as_any().downcast_ref::<VulkanSampler>().unwrap().get_resource())
                    .image_view(info.image_view.as_any().downcast_ref::<VulkanImageView>().unwrap().get_resource())
                    .image_layout(vk::ImageLayout::from_raw(info.image_layout.as_raw()))
                    .build()
            }).collect::<Vec<_>>();
            let buffer_info = write.buffer_info.iter().map(|info| {
                vk::DescriptorBufferInfo::builder()
                    .buffer(info.buffer.as_any().downcast_ref::<VulkanBuffer>().unwrap().get_resource())
                    .offset(info.offset)
                    .range(info.range)
                    .build()
            }).collect::<Vec<_>>();
            let texel_buffer_view = write.texel_buffer_view.iter().map(|view| {
                view.as_any().downcast_ref::<VulkanBufferView>().unwrap().get_resource()
            }).collect::<Vec<_>>();
            vk::WriteDescriptorSet::builder()
                .dst_set(write.dst_set.as_any().downcast_ref::<VulkanDescriptorSet>().unwrap().get_resource())
                .dst_binding(write.dst_binding)
                .dst_array_element(write.dst_array_element)
                .descriptor_type(vk::DescriptorType::from_raw(write.descriptor_type.as_raw()))
                .image_info(&image_info)
                .buffer_info(&buffer_info)
                .texel_buffer_view(&texel_buffer_view)
                .build()
        }).collect::<Vec<_>>();

        unsafe {
            self.m_device.as_ref().unwrap().update_descriptor_sets(&vk_writes, &[] as &[vk::CopyDescriptorSet]);
        }

        Ok(())
    }

    fn get_current_command_buffer(&self) -> &Box<dyn RHICommandBuffer> {
        self.m_current_command_buffer.as_ref().unwrap()
    }
    
    fn get_descriptor_pool(&self) -> Result<&Box<dyn RHIDescriptorPool>> {
        Ok(self.m_descriptor_pool.as_ref().unwrap())
    }

    fn get_swap_chain_info(&'_ self) -> RHISwapChainDesc<'_> {
        RHISwapChainDesc{
            extent: self.m_swapchain_extent,
            image_format: self.m_swapchain_image_format,
            viewport: &self.m_viewport,
            scissor: &self.m_scissor,
            image_views: &self.m_swapchain_image_views,
        }
    }

    fn get_depth_image_info(&'_ self) -> RHIDepthImageDesc<'_> {
        RHIDepthImageDesc {
            image: self.m_depth_image.as_ref().unwrap(),
            image_view: self.m_depth_image_view.as_ref().unwrap(),
            format: self.m_depth_image_format,
        }
    }
    
    fn get_max_frames_in_flight(&self) -> u8 {
        Self::K_MAX_FRAMES_IN_FLIGHT
    }

    fn get_current_frame_index(&self) -> u8 {
        self.m_current_frame_index
    }

    fn begin_single_time_commands(&self) -> Result<Box<dyn RHICommandBuffer>> {
        let command_pool = self.m_rhi_command_pool.as_ref().unwrap().as_any().downcast_ref::<VulkanCommandPool>().unwrap().get_resource();
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { self.m_device.as_ref().unwrap().allocate_command_buffers(&allocate_info)?[0] };

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.m_device.as_ref().unwrap().begin_command_buffer(command_buffer, &begin_info)?;
        }
        let command_buffer = Box::new(VulkanCommandBuffer::new(command_buffer)) as Box<dyn RHICommandBuffer>;
        Ok(command_buffer)
    }

    fn end_single_time_commands(&self, command_buffer: Box<dyn RHICommandBuffer>) -> Result<()> {
        let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
        unsafe { self.m_device.as_ref().unwrap().end_command_buffer(command_buffer)? };
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers);
        let graphics_queue = self.m_graphics_queue.as_ref().unwrap().as_any().downcast_ref::<VulkanQueue>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().queue_submit(graphics_queue, &[submit_info], vk::Fence::null())?;
            self.m_device.as_ref().unwrap().queue_wait_idle(graphics_queue)?;
            let command_pool = self.m_rhi_command_pool.as_ref().unwrap().as_any().downcast_ref::<VulkanCommandPool>().unwrap().get_resource();
            self.m_device.as_ref().unwrap().free_command_buffers(command_pool, &command_buffers);
        }
        Ok(())
    }

    fn push_event(&self, command_buffer: &Box<dyn RHICommandBuffer>, event_name: &str, color: [f32; 4]) {
        if self.m_enable_debug_utils_label {
            let label_info = vk::DebugUtilsLabelEXT::builder()
                .label_name(event_name.as_bytes())
                .color(color);
            let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
            unsafe{
                self.m_instance.as_ref().unwrap().cmd_begin_debug_utils_label_ext(command_buffer, &label_info);
            }
        }
    }

    fn pop_event(&self, command_buffer: &Box<dyn RHICommandBuffer>) {
        if self.m_enable_debug_utils_label {
            let command_buffer = command_buffer.as_any().downcast_ref::<VulkanCommandBuffer>().unwrap().get_resource();
            unsafe{
                self.m_instance.as_ref().unwrap().cmd_end_debug_utils_label_ext(command_buffer);
            }
        }
    }

    fn destroy_shader_module(&self, shader: Box<dyn RHIShader>) {
        unsafe{
            self.m_device.as_ref().unwrap().destroy_shader_module(shader.as_any().downcast_ref::<VulkanShader>().unwrap().get_resource(), None);
        }
    }
    
    fn destroy_image_view(&self, image_view: Box<dyn RHIImageView>) {
        let image_view = image_view.as_any().downcast_ref::<VulkanImageView>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().destroy_image_view(image_view, None);
        }
    }

    fn destroy_image(&self, image: Box<dyn RHIImage>) {
        let image = image.as_any().downcast_ref::<VulkanImage>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().destroy_image(image, None);
        }
    }

    fn destroy_framebuffer(&self, framebuffer: Box<dyn RHIFramebuffer>) {
        let framebuffer = framebuffer.as_any().downcast_ref::<VulkanFramebuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().destroy_framebuffer(framebuffer, None);
        }
    }

    fn destroy_buffer(&self, buffer: Box<dyn RHIBuffer>) {
        let buffer = buffer.as_any().downcast_ref::<VulkanBuffer>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().destroy_buffer(buffer, None);
        }
    }

    fn free_memory(&self, memory: Box<dyn RHIDeviceMemory>) {
        let memory = memory.as_any().downcast_ref::<VulkanDeviceMemory>().unwrap().get_resource();
        unsafe {
            self.m_device.as_ref().unwrap().free_memory(memory, None);
        }
    }

    fn map_memory(&self, memory: &Box<dyn RHIDeviceMemory>, offset: RHIDeviceSize, size: RHIDeviceSize, flags: RHIMemoryMapFlags) -> Result<*mut std::ffi::c_void> {
        Ok(unsafe{self.m_device.as_ref().unwrap().map_memory(
            memory.as_any().downcast_ref::<VulkanDeviceMemory>().unwrap().get_resource(), 
            offset, 
            size, 
            vk::MemoryMapFlags::from_bits_unchecked(flags.bits()), 
        )?})
    }

    fn unmap_memory(&self, memory: &Box<dyn RHIDeviceMemory>) {
        unsafe{self.m_device.as_ref().unwrap().unmap_memory(
            memory.as_any().downcast_ref::<VulkanDeviceMemory>().unwrap().get_resource(), 
        )}
    }
}

impl VulkanRHI {
    pub const K_MAX_FRAMES_IN_FLIGHT: u8 = 3;

    pub fn extracted(&self, chosen_extent: vk::Extent2D){
        
    }
}

impl VulkanRHI {

    fn check_validation_layer_support(&self) -> Result<bool> {
        let available_layers = unsafe {
            self.m_entry.as_ref().unwrap()
            .enumerate_instance_layer_properties()?
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>()
        }; 

        let res = self.m_validation_layers.iter().all(|&layer_name|
            available_layers.contains(&layer_name)
        );

        Ok(res)
    }
    
    fn get_required_extensions(&self) -> Result<Vec<*const i8>> {

        let binding = self.m_window.upgrade();
        let window = binding.as_ref().unwrap();

        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        if self.m_enable_validation_layers || self.m_enable_debug_utils_label {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        if cfg!(target_os = "macos") && self.m_entry.as_ref().unwrap().version()? >= PORTABILITY_MACOS_VERSION {
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

    fn populate_debug_messenger_info(info: &mut vk::DebugUtilsMessengerCreateInfoEXTBuilder<'_>) {
        info
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            )
            .user_callback(Some(Self::debug_callback));
    }
    
    fn create_instance(&mut self) -> Result<()> {
        if self.m_enable_validation_layers && !self.check_validation_layer_support()? {
            error!("validation layers requested, but not available!");
        }

        self.m_vulkan_api_version = vk::make_version(1, 0, 0);

        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Vulkan Example\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(self.m_vulkan_api_version);

        let extensions = self.get_required_extensions()?;

        let flags = if cfg!(target_os = "macos") && self.m_entry.as_ref().unwrap().version()? >= PORTABILITY_MACOS_VERSION{
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let validation_layers = self.m_validation_layers.iter().map(|&layer| layer.as_ptr()).collect::<Vec<_>>();

        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&validation_layers)
            .enabled_extension_names(&extensions)
            .flags(flags);

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder();
        Self::populate_debug_messenger_info(&mut debug_info);

        if self.m_enable_validation_layers {
            info = info.push_next(&mut debug_info);
        }

        self.m_instance = Some(unsafe {
            self.m_entry.as_ref().unwrap().create_instance(&info, None)?
        });

        Ok(())
    }

    fn initialize_debug_messenger(&mut self) -> Result<()> {
        if self.m_enable_validation_layers {
            let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder();
            Self::populate_debug_messenger_info(&mut debug_info);
            self.m_debug_messenger = unsafe {
                self.m_instance.as_ref().unwrap().create_debug_utils_messenger_ext(&debug_info, None)?
            };
        }

        Ok(())
    }

    fn create_window_surface(&mut self) -> Result<()> {
        unsafe {
            let window = self.m_window.upgrade().unwrap();
            self.m_surface = vk_window::create_surface(self.m_instance.as_ref().unwrap(), &window, &window)?;
        }
        Ok(())
    }

    fn find_queue_families(&self, physical_device: vk::PhysicalDevice) -> Result<QueueFamilyIndices> {
        let properties = unsafe {
            self.m_instance.as_ref().unwrap().get_physical_device_queue_family_properties(physical_device)
        };
        let mut indices = QueueFamilyIndices::default();
        for (index,properties) in properties.iter().enumerate() {
            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) { 
                indices.graphics_family = Some(index as u32);
            }
            if unsafe {self.m_instance.as_ref().unwrap().get_physical_device_surface_support_khr(physical_device, index as u32, self.m_surface)?} {
                indices.present_family = Some(index as u32);
            }
            if indices.is_complete() {
                break;
            }
        }
        Ok(indices)
    }

    fn check_device_extension_support(&self, physical_device: vk::PhysicalDevice) -> Result<bool> {
        let extensions = unsafe {
            self.m_instance.as_ref().unwrap()
                .enumerate_device_extension_properties(physical_device, None)?
                .iter()
                .map(|e| e.extension_name)
                .collect::<HashSet<_>>()
        };

        Ok(self.m_device_extensions.iter().all(|e| extensions.contains(e)))
    }
    
    fn query_swapchain_support(&self, physical_device: vk::PhysicalDevice) -> Result<SwapChainSupportDetails> { 
        unsafe {
            Ok(SwapChainSupportDetails { 
                capabilities: self.m_instance.as_ref().unwrap().
                    get_physical_device_surface_capabilities_khr(physical_device, self.m_surface)?,
                formats: self.m_instance.as_ref().unwrap().
                    get_physical_device_surface_formats_khr(physical_device, self.m_surface)?,
                present_modes: self.m_instance.as_ref().unwrap().
                    get_physical_device_surface_present_modes_khr(physical_device, self.m_surface)?,
            })
        }
    }
    
    fn is_device_suitable(&self, physical_device: vk::PhysicalDevice) -> Result<bool> { 
        let queue_indices = self.find_queue_families(physical_device)?;
        if !queue_indices.is_complete() {
            return Ok(false);
        }
        if !self.check_device_extension_support(physical_device)? {
            return Ok(false);
        }
        let swapchain_support_details = self.query_swapchain_support(physical_device)?;
        let is_swapchain_adequate =
            !swapchain_support_details.formats.is_empty() && !swapchain_support_details.present_modes.is_empty();

        if !is_swapchain_adequate {
            return Ok(false);
        }

        let features = unsafe{ self.m_instance.as_ref().unwrap().get_physical_device_features(physical_device) };
        if features.sampler_anisotropy != vk::TRUE {
            return Ok(false);
        }

        Ok(true)
    }

    fn initial_physical_device(&mut self) -> Result<()>{
        let mut ranked_physical_devices = vec![];
        unsafe {
            for physical_device in self.m_instance.as_ref().unwrap().enumerate_physical_devices()?{
                let properties = self.m_instance.as_ref().unwrap().get_physical_device_properties(physical_device);
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

        self.m_device_extensions = vec![vk::KHR_SWAPCHAIN_EXTENSION.name];

        for (physical_device, _) in ranked_physical_devices {
            if (self.is_device_suitable(physical_device))? {
                self.m_physical_device = physical_device;
                return Ok(());
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }

    fn find_supported_format(&self, candidates: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> Result<vk::Format> {
        for &format in candidates {
            let properties = unsafe {
                self.m_instance.as_ref().unwrap().get_physical_device_format_properties(self.m_physical_device, format)
            };
            if tiling == vk::ImageTiling::LINEAR && properties.linear_tiling_features.contains(features) {
                return Ok(format);
            } else if tiling == vk::ImageTiling::OPTIMAL && properties.optimal_tiling_features.contains(features) {
                return Ok(format);
            }
        }
        Err(anyhow!("Failed to find supported format."))
    }

    fn find_depth_format(&self) -> Result<vk::Format> {
        self.find_supported_format(
            &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT], 
            vk::ImageTiling::OPTIMAL, 
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
        )
    }

    fn create_logical_device(&mut self) -> Result<()> {
        self.m_queue_indices = self.find_queue_families(self.m_physical_device)?;

        let mut unique_indices = HashSet::new();
        unique_indices.insert(self.m_queue_indices.graphics_family.unwrap());
        unique_indices.insert(self.m_queue_indices.present_family.unwrap());


        let queue_priorities = &[1.0];
        let queue_infos = unique_indices.
            iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
            })
            .collect::<Vec<_>>();

        let layers = self.m_validation_layers.iter().map(|&layer| layer.as_ptr()).collect::<Vec<_>>();
        let mut extensions = self.m_device_extensions.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();
        if cfg!(target_os = "macos") && self.m_entry.as_ref().unwrap().version()? >= PORTABILITY_MACOS_VERSION {
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
            self.m_instance.as_ref().unwrap().create_device(self.m_physical_device, &device_info, None)?
        };

        unsafe {
            let vk_graphics_queue = device.get_device_queue(self.m_queue_indices.graphics_family.unwrap(), 0);
            self.m_graphics_queue = Some(Box::new(VulkanQueue::new(vk_graphics_queue)));
            self.m_present_queue = device.get_device_queue(self.m_queue_indices.present_family.unwrap(), 0);
        }

        self.m_depth_image_format = RHIFormat::from_raw(self.find_depth_format()?.as_raw());

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

    fn create_command_pool(&mut self) -> Result<()>{

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(self.m_queue_indices.graphics_family.unwrap());

        let command_pool = unsafe { self.m_device.as_ref().unwrap().create_command_pool(&info, None)? };
        self.m_rhi_command_pool = Some(Box::new(VulkanCommandPool::new(command_pool)));

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(self.m_queue_indices.graphics_family.unwrap());

        for i in 0..Self::K_MAX_FRAMES_IN_FLIGHT as usize {
            self.m_command_pools[i] = unsafe { self.m_device.as_ref().unwrap().create_command_pool(&info, None)? };
        }

        Ok(())
    }

    fn create_command_buffers(&mut self) -> Result<()> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        self.m_command_buffers = Some(array::from_fn(|i| {
            allocate_info.command_pool(self.m_command_pools[i]);
            self.m_vk_command_buffers[i] = unsafe {
                self.m_device.as_ref().unwrap().allocate_command_buffers(&allocate_info).unwrap()
            }[0];
            Box::new(VulkanCommandBuffer::new(self.m_vk_command_buffers[i])) as Box<dyn RHICommandBuffer>
        }));
        Ok(())
    }

    fn create_descriptor_pool(&mut self) -> Result<()> {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .descriptor_count(3 + 2 + 2 + 2 + 1 + 1 + 3 + 3),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1 + 1 + 1 * self.m_max_vertex_blending_mesh_count),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1 * self.m_max_material_count),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(3 + 5 * self.m_max_material_count + 1 + 1),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(4 + 1 + 1 + 2),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(3),
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1),
        ];

        let info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1 + 1 + 1 + self.m_max_material_count + self.m_max_vertex_blending_mesh_count + 1 + 1);

        unsafe {
            self.m_vk_descriptor_pool = self.m_device.as_ref().unwrap().create_descriptor_pool(&info, None)?;
        }

        self.m_descriptor_pool = Some(Box::new(VulkanDescriptorPool::new(self.m_vk_descriptor_pool)));

        Ok(())
    }

    fn create_sync_primitives(&mut self) -> Result<()> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        unsafe {
            self.m_image_available_for_texturescopy_semaphores = Some(array::from_fn(|i| {
                let device = self.m_device.as_ref().unwrap();
                self.m_image_available_for_render_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap();
                self.m_image_finished_for_presentation_semaphores[i] = device.create_semaphore(&semaphore_info, None).unwrap();
                let semaphore = device.create_semaphore(&semaphore_info, None).unwrap();
                
                Box::new(VulkanSemaphore::new(semaphore)) as Box<dyn RHISemaphore>
            }));

            self.m_rhi_is_frame_in_flight_fences = Some(array::from_fn(|i| {
                let device = self.m_device.as_ref().unwrap();
                self.m_is_frame_in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap();
                Box::new(VulkanFence::new(self.m_is_frame_in_flight_fences[i])) as Box<dyn RHIFence>
            }));
        }

        Ok(())
    }

    fn create_framebuffer_image_and_views(&mut self) -> Result<()> {
        let (image, image_memory) = VulkanUtil::create_image(
            self.m_instance.as_ref().unwrap(),
            self.m_device.as_ref().unwrap(),
            self.m_physical_device,
            self.m_swapchain_extent.width,
            self.m_swapchain_extent.height,
            vk::Format::from_raw(self.m_depth_image_format.as_raw()),
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageCreateFlags::empty(),
            1,
            1,
        )?;

        self.m_depth_image = Some(Box::new(VulkanImage::new(image)));
        self.m_depth_image_memory = image_memory;

        let image_view = VulkanUtil::create_image_view(
            self.m_device.as_ref().unwrap(),
            image,
            vk::Format::from_raw(self.m_depth_image_format.as_raw()),
            vk::ImageAspectFlags::DEPTH,
            vk::ImageViewType::_2D,
            1,
            1,
        )?;

        self.m_depth_image_view = Some(Box::new(VulkanImageView::new(image_view)));

        Ok(())
    }

    fn create_assert_allocator(&mut self) -> Result<()> {
        //todo: create allocator
        Ok(())
    }
}