use std::{fs::File, io::BufReader, ptr::copy_nonoverlapping as memcpy};
use anyhow::{anyhow, Result};
use vulkanalia::{prelude::v1_0::*};

use crate::vulkan::{begin_single_time_commands, copy_buffer_to_image, create_buffer, create_image, create_image_view, resource_manager::ResourceManager, transition_image_layout, utils::end_single_time_commands, Destroy, VulkanData};

#[derive(Clone, Debug, Default)]
pub struct Texture{
    pub _mip_levels: u32,
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
}

impl Texture {
    pub fn new(instance: &Instance, device: &Device, data: &mut VulkanData, file_path: &str) -> Result<Self> {
        let (image, image_memory, mip_levels) = create_texture_image(instance, device, data, file_path)?;
        let image_view = create_image_view(device, image, vk::Format::R8G8B8A8_SRGB,vk::ImageAspectFlags::COLOR, mip_levels)?;
        Ok(Texture{_mip_levels:mip_levels,image,image_memory,image_view})
    }

    pub fn destroy(&self, device: &Device){
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.image_memory, None);
        }
    }
}

fn create_texture_image(instance: &Instance, device: &Device, data: &mut VulkanData, file_path: &str) -> Result<(vk::Image, vk::DeviceMemory, u32)> {

    let image = File::open(file_path)?;

    let decoder = png::Decoder::new(BufReader::new(image));
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();
    let mip_levels = (width.max(height) as f32).log2().floor() as u32 +1;

    let memory_properties = unsafe{instance.get_physical_device_memory_properties(data.physical_device)};

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        device, &memory_properties, size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
        memcpy(pixels.as_ptr(), memory.cast(), pixels.len());
        device.unmap_memory(staging_buffer_memory);
    }

    let (texture_image, texture_image_memory) = create_image(
        instance, device, data, width, height, mip_levels,
        vk::SampleCountFlags::_1,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    transition_image_layout(
        device, data, texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels
    )?;

    copy_buffer_to_image(device, data, staging_buffer, texture_image, width, height)?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    generate_mipmaps(instance, device, data, texture_image,vk::Format::R8G8B8A8_SRGB, width, height, mip_levels)?;

    Ok((texture_image, texture_image_memory, mip_levels))
}

fn generate_mipmaps(
    instance: &Instance,
    device: &Device,
    data: &VulkanData,
    image: vk::Image,
    format: vk::Format,
    width: u32,
    height: u32,
    mip_levels: u32
) -> Result<()> {

    unsafe {
        if !instance
            .get_physical_device_format_properties(data.physical_device, format)
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            return Err(anyhow!("Texture image format does not support linear blitting!"));
        }
    }
    

    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(1)
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
            device.cmd_pipeline_barrier(
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
            .layer_count(1);

        let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i)
            .base_array_layer(0)
            .layer_count(1);

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
            device.cmd_blit_image(
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
            device.cmd_pipeline_barrier(
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
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }
   

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub type TextureManager = ResourceManager<Texture>;

impl Destroy for TextureManager {
    fn destroy(&mut self, device: &Device) {
        for texture in self.values_mut() {
            texture.destroy(device);
        }
        self.clear();
    }
}
