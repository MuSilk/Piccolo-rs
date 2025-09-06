
use std::{ptr::copy_nonoverlapping as memcpy};
use anyhow::{Result};
use vulkanalia::{prelude::v1_0::*};

use crate::vulkan::{copy_buffer, create_buffer, VulkanData};

#[derive(Clone, Debug, Default)]
pub struct VertexObject{
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
}

impl VertexObject {
    pub fn new<Vertex>(instance: &Instance, device: &Device, data: &VulkanData, vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Result<Self> {

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(instance, device, data, vertices)?;
        let (index_buffer, index_buffer_memory) = create_index_buffer(instance, device, data, indices)?;

        Ok(Self {
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_buffer(self.vertex_buffer, None);
            device.free_memory(self.vertex_buffer_memory, None);
            device.destroy_buffer(self.index_buffer, None);
            device.free_memory(self.index_buffer_memory, None);
        }
    }
}

fn create_vertex_buffer<Vertex>(instance: &Instance, device: &Device, data: &VulkanData, vertices: &Vec<Vertex>) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let memory_properties = unsafe{instance.get_physical_device_memory_properties(data.physical_device)};

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        device,
        &memory_properties,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let memory: *mut std::ffi::c_void = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
        memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
        device.unmap_memory(staging_buffer_memory);
    }

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        device, 
        &memory_properties, 
        size, 
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Ok((vertex_buffer, vertex_buffer_memory))
}

fn create_index_buffer(instance: &Instance, device: &Device, data: &VulkanData, indices: &Vec<u32>) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (size_of::<u32>() * indices.len()) as u64;

    let memory_properties = unsafe{instance.get_physical_device_memory_properties(data.physical_device)};

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        device,
        &memory_properties,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
        memcpy(indices.as_ptr(), memory.cast(), indices.len());
        device.unmap_memory(staging_buffer_memory);
    }

    let (index_buffer, index_buffer_memory) = create_buffer(
        device,
        &memory_properties,
        size,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Ok((index_buffer, index_buffer_memory))
}
