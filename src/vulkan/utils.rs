use anyhow::{anyhow,Result};
use vulkanalia::{prelude::v1_0::*, vk::PhysicalDeviceMemoryProperties};

use crate::vulkan::VulkanData;

pub trait Destroy {
    fn destroy(&mut self, device: &Device);
}

pub fn get_memory_type_index(
    memory_properties: &PhysicalDeviceMemoryProperties,
    properties: vk::MemoryPropertyFlags, 
    requirements: vk::MemoryRequirements) 
    -> Result<u32> 
{
    (0..memory_properties.memory_type_count)
        .find(|i|{
            let suitable = (requirements.memory_type_bits & (1<<i)) !=0;
            let memory_type = memory_properties.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

pub fn create_buffer(
    device: &Device,
    memory_properties: &PhysicalDeviceMemoryProperties,
    size: u64,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .flags(vk::BufferCreateFlags::empty());

    unsafe {
        let buffer = device.create_buffer(&buffer_info, None)?;
        let requirements = device.get_buffer_memory_requirements(buffer);
        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(get_memory_type_index(
                memory_properties,
                properties,
                requirements,
            )?);

        let buffer_memory = device.allocate_memory(&memory_info, None)?;
        device.bind_buffer_memory(buffer, buffer_memory, 0)?;
        Ok((buffer, buffer_memory))
    }
}

pub fn copy_buffer(
    device: &Device,
    data: &VulkanData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {

    let command_buffer = begin_single_time_commands(device, data)?;

    unsafe {
        let regions = [vk::BufferCopy::builder().size(size).build()];
        device.cmd_copy_buffer(command_buffer, source, destination, &regions);
    }

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub fn create_image(
    instance: &Instance,
    device: &Device,
    data: &mut VulkanData,
    width: u32,
    height: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D { width, height, depth: 1 })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples)
        .flags(vk::ImageCreateFlags::empty());

    unsafe {
        let image = device.create_image(&info, None)?;
        let requirements = device.get_image_memory_requirements(image);
        let memory_properties = instance.get_physical_device_memory_properties(data.physical_device);
        let info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(get_memory_type_index(
                &memory_properties,
                properties,
                requirements,
            )?);

        let image_memory = device.allocate_memory(&info, None)?;
        device.bind_image_memory(image, image_memory, 0)?;
        Ok((image, image_memory))
    }
}

pub fn create_image_view(device: &Device,image: vk::Image,format: vk::Format,aspects: vk::ImageAspectFlags,mip_levels: u32) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(unsafe {
        device.create_image_view(&info, None)?
    })
}

pub fn transition_image_layout(
    device: &Device,
    data: &VulkanData,
    image: vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32
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
        _ => return Err(anyhow!("Unsupported image layout transition!")),
    };

    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

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
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub fn copy_buffer_to_image(
    device: &Device,
    data: &VulkanData,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub fn begin_single_time_commands(device: &Device, data: &VulkanData) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info)?[0] };

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(command_buffer, &info)?;
    }

    Ok(command_buffer)
}

pub fn end_single_time_commands(device: &Device, data: &VulkanData, command_buffer: vk::CommandBuffer) -> Result<()> {
    unsafe {
        device.end_command_buffer(command_buffer)?;
    }
    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);
    
    unsafe {
        device.queue_submit(data.graphics_queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(data.graphics_queue)?;
        device.free_command_buffers(data.command_pool, &[command_buffer]);
    }

    Ok(())
}



