use std::{ptr::copy_nonoverlapping as memcpy};
use anyhow::{Result};
use nalgebra_glm::Mat4;
use vulkanalia::{bytecode::Bytecode, prelude::v1_0::*};

use crate::{surface::VertexLayout, vulkan::{create_buffer, resource_manager::ResourceManager, Destroy, VulkanData}};

#[derive(Clone, Debug, Default)]
pub struct Pipeline{
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    swapchain_image_count: usize,
}

impl Pipeline {
    pub fn create_pipeline<Vertex: VertexLayout>(device: &Device, instance: &Instance, data: &VulkanData) -> Result<Pipeline> {

        let swapchain_image_count = data.swapchain_images.len() as u32;

        let descriptor_set_layout = create_descriptor_set_layout(device)?;
        let descriptor_pool = create_descriptor_pool(device, swapchain_image_count)?;
        let descriptor_sets = create_descriptor_sets(device, descriptor_set_layout, descriptor_pool, swapchain_image_count as usize)?;

        let set_layouts = &[descriptor_set_layout];
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts);
        let pipeline_layout = unsafe {device.create_pipeline_layout(&layout_info, None)?};

        let pipeline = create_pipeline::<Vertex>(device, data.render_pass, data.swapchain_extent, data.msaa_samples, pipeline_layout)?;
        let memory_properties = data.get_memory_properties(instance);
        let (uniform_buffers,uniform_buffers_memory) = create_uniform_buffers(device, &memory_properties, swapchain_image_count)?;

        Ok(Pipeline{
            pipeline, pipeline_layout, 
            descriptor_set_layout, descriptor_pool,descriptor_sets,
            uniform_buffers,uniform_buffers_memory,
            swapchain_image_count : swapchain_image_count as usize
        })
    }
    pub fn recreate<Vertex: VertexLayout>(
        &mut self,
        device: &Device,
        instance: &Instance,
        data: &VulkanData,
    ) -> Result<()>{
        unsafe {
            let swapchain_image_count = data.swapchain_images.len() as u32;
            let memory_properties = data.get_memory_properties(instance);
            self.swapchain_image_count = swapchain_image_count as usize;

            while self.uniform_buffers.len() < swapchain_image_count as usize {
                let (uniform_buffer, uniform_buffer_memory) = create_buffer(
                    device, &memory_properties,
                    size_of::<UniformBufferObject>() as u64,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
                )?;
                self.uniform_buffers.push(uniform_buffer);
                self.uniform_buffers_memory.push(uniform_buffer_memory);
            }
            while self.uniform_buffers.len() > swapchain_image_count as usize {
                if let Some(uniform_buffer) = self.uniform_buffers.pop() {
                    device.destroy_buffer(uniform_buffer, None);
                }
                if let Some(uniform_buffer_memory) = self.uniform_buffers_memory.pop() {
                    device.free_memory(uniform_buffer_memory, None);
                }
            }
            
            let new_descriptor_pool = create_descriptor_pool(device, swapchain_image_count)?;
            let new_descriptor_sets = create_descriptor_sets(device, self.descriptor_set_layout, new_descriptor_pool, swapchain_image_count as usize)?;
            
            for i in 0..swapchain_image_count as usize {
                let info = vk::DescriptorBufferInfo::builder()
                    .buffer(self.uniform_buffers[i])
                    .offset(0)
                    .range(std::mem::size_of::<UniformBufferObject>() as u64);

                let buffer_info = &[info];
                let ubo_write = vk::WriteDescriptorSet::builder()
                    .dst_set(new_descriptor_sets[i])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(buffer_info);

                let sampler_copy = vk::CopyDescriptorSet::builder()
                    .src_set(self.descriptor_sets[0])
                    .src_binding(1)
                    .dst_set(new_descriptor_sets[i])
                    .dst_binding(1)
                    .src_array_element(0)
                    .dst_array_element(0)
                    .descriptor_count(1);

                device.update_descriptor_sets(&[ubo_write], &[sampler_copy]);
            }
            
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.descriptor_pool = new_descriptor_pool;
            self.descriptor_sets = new_descriptor_sets;
            device.destroy_pipeline(self.pipeline, None);
            self.pipeline = create_pipeline::<Vertex>(device, data.render_pass, data.swapchain_extent, data.msaa_samples, self.pipeline_layout)?;
        } 
        Ok(())
    }
    pub fn destroy(&self, device: &Device){
        unsafe {
            self.uniform_buffers.iter().for_each(|b| device.destroy_buffer(*b, None));
            self.uniform_buffers_memory.iter().for_each(|m| device.free_memory(*m, None));
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
    pub fn set_uniform<T>(&self, device: &Device, index: usize, data:  *const T) -> Result<()>{
        unsafe {
            let memory = device.map_memory(
                self.uniform_buffers_memory[index],
                0,
                size_of::<UniformBufferObject>() as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            memcpy(data, memory.cast(), 1);

            device.unmap_memory(self.uniform_buffers_memory[index]);
        }    
        Ok(())
    }
    pub fn update_descriptor_sets(
        &self, device: &Device, 
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Result<()> {
        for i in 0..self.swapchain_image_count {
            let info = vk::DescriptorBufferInfo::builder()
                .buffer(self.uniform_buffers[i])
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let buffer_info = &[info];
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            let info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_image_view)
                .sampler(texture_sampler);

            let image_info = &[info];
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_info);

            unsafe {
                device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
            }
        }
        Ok(())
    }
}

fn create_pipeline<Vertex: VertexLayout>(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    pipeline_layout: vk::PipelineLayout,
) -> Result<vk::Pipeline> {

    let vert = include_bytes!("../../shaders/shader.vert.spv");
    let frag = include_bytes!("../../shaders/shader.frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(msaa_samples);

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.0)
        .max_depth_bounds(1.0)
        .stencil_test_enable(false);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_handle(vk::Pipeline::null())
        .base_pipeline_index(-1);

    let pipeline = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?.0[0]
    };

    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    }

    Ok(pipeline)
}

fn create_shader_module(device: &Device,bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    unsafe{Ok(device.create_shader_module(&info, None)?)}
}

fn create_descriptor_pool(device: &Device, descriptor_count: u32) -> Result<vk::DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(descriptor_count);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(descriptor_count);

    let pool_sizes = [ubo_size, sampler_size];

    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(descriptor_count);

    unsafe {
        Ok(device.create_descriptor_pool(&info, None)?)
    }
}

fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_bindings = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = [ubo_bindings, sampler_binding];

    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings);

    unsafe {
        Ok(device.create_descriptor_set_layout(&info, None)?)
    }
}

fn create_descriptor_sets(
    device: &Device, 
    descriptor_layout: vk::DescriptorSetLayout, 
    descriptor_pool: vk::DescriptorPool,
    descriptor_count: usize
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![descriptor_layout; descriptor_count];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    Ok(unsafe {device.allocate_descriptor_sets(&info)?})
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}
fn create_uniform_buffers(
    device: &Device,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    buffer_count: u32) 
    -> Result<(Vec<vk::Buffer>,Vec<vk::DeviceMemory>)> 
{
    let mut uniform_buffers: Vec<vk::Buffer> = Vec::new();
    let mut uniform_buffers_memory: Vec<vk::DeviceMemory>  = Vec::new();

    for _ in 0..buffer_count {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            device, memory_properties,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        )?;

        uniform_buffers.push(uniform_buffer);
        uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok((uniform_buffers, uniform_buffers_memory))
}

pub type PipelineManager = ResourceManager<Pipeline>;

impl Destroy for PipelineManager {
    fn destroy(&mut self, device: &Device) {
        for data in self.values_mut() {
            data.destroy(device);
        }
        self.clear();
    }
}