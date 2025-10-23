use std::ptr::copy_nonoverlapping;

use anyhow::{anyhow, Result};
use vulkanalia::{bytecode::Bytecode, prelude::v1_0::*};

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;

pub fn find_memory_type(
    instance: &Instance,
    physical_device: vk::PhysicalDevice, 
    type_filter: u32, 
    properties_flag: vk::MemoryPropertyFlags
) -> Result<u32>{
    let physical_device_memory_properties = unsafe {
        instance.get_physical_device_memory_properties(physical_device)
    };

    for (i, memory_type) in physical_device_memory_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i) != 0) && memory_type.property_flags.contains(properties_flag) {
            return Ok(i as u32);
        }
    }

    Err(anyhow::anyhow!("Failed to find suitable memory type"))
}

pub fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();

    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(bytecode.code())
        .code_size(bytecode.code_size());
    Ok(unsafe { device.create_shader_module(&create_info, None)? })
}

pub fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size) 
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    unsafe {
        let buffer = device.create_buffer(&buffer_info, None)?;
        let requirements = device.get_buffer_memory_requirements(buffer);
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(
                find_memory_type(instance, physical_device, requirements.memory_type_bits, properties)?
            );

        let buffer_memory = device.allocate_memory(&info, None)?;
        device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        Ok((buffer, buffer_memory))
    }
}

pub fn copy_buffer(
    rhi: &VulkanRHI,
    device: &Device,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    src_offset: vk::DeviceSize,
    dst_offset: vk::DeviceSize,
    size: vk::DeviceSize
) -> Result<()> {

    let command_buffer = rhi.begin_single_time_commands()?;

    let copy_region = vk::BufferCopy::builder()
        .size(size)
        .src_offset(src_offset)
        .dst_offset(dst_offset);

    unsafe {
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region.build()]);
    }

    rhi.end_single_time_commands(command_buffer)?;

    Ok(())
}

fn copy_buffer_to_image(
    rhi: &VulkanRHI,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
    layout_count: u32,
) -> Result<()> { 
    let command_buffer = rhi.begin_single_time_commands()?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(layout_count);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });

    unsafe {
        rhi.m_device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    rhi.end_single_time_commands(command_buffer)?;
    Ok(())
}

fn transition_image_layout(
    rhi: &VulkanRHI,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    layer_count: u32,
    mip_levels: u32,
    aspect_mask_bits: vk::ImageAspectFlags,
) -> Result<()> {

    let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask,
    ) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        (vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL) => (
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::AccessFlags::TRANSFER_READ,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_READ,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::TRANSFER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
        ),
        _ => return Err(anyhow!("Unsupported image layout transition!")),
    };

    let command_buffer = rhi.begin_single_time_commands()?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask_bits)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(layer_count);

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    unsafe {
        rhi.m_device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }

    rhi.end_single_time_commands(command_buffer)?;
    Ok(())
}

fn generate_mipmaps(
    rhi: &VulkanRHI,
    image: vk::Image,
    format: vk::Format,
    width: u32,
    height: u32,
    layers: u32,
    mip_levels: u32
) -> Result<()> {

    unsafe {
        if !rhi.m_instance
            .get_physical_device_format_properties(rhi.m_data.m_physical_device, format)
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            return Err(anyhow!("Texture image format does not support linear blitting!"));
        }
    }

    let command_buffer = rhi.begin_single_time_commands()?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(layers)
        .level_count(1);

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(subresource);

    let mut mip_width = width;
    let mut mip_height = height;

    for i in 1..mip_levels {
        barrier.subresource_range.base_mip_level = i-1;
        barrier.old_layout=vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout=vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.src_access_mask=vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask=vk::AccessFlags::TRANSFER_READ;
        unsafe {
            rhi.m_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        }

        let src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i-1)
            .base_array_layer(0)
            .layer_count(layers);

        let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i)
            .base_array_layer(0)
            .layer_count(layers);

        let blit = vk::ImageBlit::builder()
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width as i32,
                    y: mip_height as i32,
                    z: 1,
                },
            ])
            .src_subresource(src_subresource)
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                    y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                    z: 1,
                },
            ])
            .dst_subresource(dst_subresource);
        unsafe {
            rhi.m_device.cmd_blit_image(
                command_buffer, 
                image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 
                image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
                &[blit],vk::Filter::LINEAR
            );
        }

        barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            rhi.m_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        }

        if mip_width>1{
            mip_width/=2;
        }
        if mip_height>1{
            mip_height/=2;
        }   
    }

    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    unsafe {
        rhi.m_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }

    rhi.end_single_time_commands(command_buffer)?;

    Ok(())
}

pub fn create_cube_map(
    rhi: &VulkanRHI, 
    width: u32, 
    height: u32, 
    pixels: &[&[u8]; 6], 
    format: vk::Format, 
    mip_levels: u32
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {

    let device = &rhi.m_device;
    let instance = &rhi.m_instance;
    let physical_device = rhi.m_data.m_physical_device;

    let cube_byte_size = pixels.iter().map(|p|p.len()).sum::<usize>();

    let image_create_info = vk::ImageCreateInfo::builder()
        .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(6)
        .format(format)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    unsafe {
        let image = device.create_image(&image_create_info, None)?;
        let requirements = device.get_image_memory_requirements(image);
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(
                find_memory_type(instance, physical_device, requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?
            );

        let image_memory = device.allocate_memory(&info, None)?;
        device.bind_image_memory(image, image_memory, 0)?;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance, device, physical_device, cube_byte_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let data = device.map_memory(staging_buffer_memory, 0, cube_byte_size as u64, vk::MemoryMapFlags::empty())?;
        let mut offset = 0;
        pixels.iter().for_each(| pixels| {
            copy_nonoverlapping(pixels.as_ptr().cast(), data.add(offset), pixels.len());
            offset += pixels.len();
        });
        device.unmap_memory(staging_buffer_memory);
        transition_image_layout(
            rhi, image, 
            vk::ImageLayout::UNDEFINED, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
            6, 
            mip_levels,
            vk::ImageAspectFlags::COLOR,
        )?;
        copy_buffer_to_image(
            rhi, 
            staging_buffer, 
            image, 
            width, 
            height,
            6
        )?;
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        generate_mipmaps(rhi, image, format, width, height, 6, mip_levels)?;
        let image_view = create_image_view(
            device, image, format, 
            vk::ImageAspectFlags::COLOR,
            vk::ImageViewType::CUBE, 
            6, 
            mip_levels
        )?;
        Ok((image, image_memory, image_view))
    }
}

pub fn create_texture_image(
    rhi: &VulkanRHI,
    width: u32,
    height: u32,
    pixels: &[u8],
    format: vk::Format,
    mip_levels: u32
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {

    let physical_device = rhi.m_data.m_physical_device;
    let size = pixels.len() as u64;
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &rhi.m_instance, &rhi.m_device, physical_device, size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let memory = rhi.m_device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
        copy_nonoverlapping(pixels.as_ptr().cast(), memory, pixels.len());
        rhi.m_device.unmap_memory(staging_buffer_memory);
    }

    let mip_levels = if mip_levels > 0 { 
        mip_levels 
    } else { 
        ((width.max(height) as f32).log2().floor() as u32) + 1 
    };

    let (texture_image, texture_image_memory) = create_image(
        &rhi.m_instance, &rhi.m_device, physical_device, width, height, format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageCreateFlags::empty(),
        1,
        mip_levels,
    )?;

    transition_image_layout(
        rhi, texture_image, 
        vk::ImageLayout::UNDEFINED, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
        1, 
        mip_levels,
        vk::ImageAspectFlags::COLOR,
    )?;

    copy_buffer_to_image(rhi, staging_buffer, texture_image, width, height, 1)?;

    unsafe {
        rhi.m_device.destroy_buffer(staging_buffer, None);
        rhi.m_device.free_memory(staging_buffer_memory, None);
    }

    generate_mipmaps(rhi, texture_image, format, width, height, 1, mip_levels)?;

    let image_view = create_image_view(
        &rhi.m_device, 
        texture_image, 
        format, 
        vk::ImageAspectFlags::COLOR, 
        vk::ImageViewType::_2D, 
        1,
        mip_levels
    )?;

    Ok((texture_image, texture_image_memory, image_view))
}
    
pub fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    image_width: u32,
    image_height: u32,
    format: vk::Format,
    image_tiling: vk::ImageTiling,
    image_usage_flags: vk::ImageUsageFlags,
    memory_property_flags: vk::MemoryPropertyFlags,
    image_create_flags: vk::ImageCreateFlags,
    array_layers: u32,
    mip_levels: u32,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D { width: image_width, height: image_height, depth: 1 })
        .mip_levels(mip_levels)
        .array_layers(array_layers)
        .format(format)
        .tiling(image_tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(image_usage_flags)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::_1)
        .flags(image_create_flags);

    unsafe {
        let image = device.create_image(&info, None)?;
        let requirements = device.get_image_memory_requirements(image);
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(
                find_memory_type(instance, physical_device, requirements.memory_type_bits, memory_property_flags)?
            );

        let image_memory = device.allocate_memory(&info, None)?;
        device.bind_image_memory(image, image_memory, 0)?;
        Ok((image, image_memory))
    }
}

pub fn create_image_view(
    device: &Device, 
    image: vk::Image, 
    format: vk::Format,
    image_aspect_flags: vk::ImageAspectFlags,
    view_type: vk::ImageViewType,
    layout_count: u32,
    mip_levels: u32
) -> Result<vk::ImageView>{
    
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(image_aspect_flags)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(layout_count);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(view_type)
        .format(format)
        .subresource_range(subresource_range);

    Ok(unsafe {device.create_image_view(&info, None)?})
}
