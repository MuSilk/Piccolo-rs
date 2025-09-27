use std::{cell::RefCell, rc::{Rc, Weak}};

use anyhow::{anyhow, Result};
use nalgebra_glm::{Mat4, Quat, Vec3};
use vulkanalia::prelude::v1_0::*;

use crate::{surface::{Mesh, TexturedMeshVertex}, vulkan::{resource_manager::ResourceManager, Pipeline, VulkanData}};

#[derive(Debug)]
pub struct RenderInstance{
    position: Vec3,
    scale: Vec3,
    rotation: Quat,

    mesh : Weak<RefCell<Mesh<TexturedMeshVertex>>>,
    command_buffer: Vec<vk::CommandBuffer>,
}

impl RenderInstance {
    pub fn new(mesh: &Rc<RefCell<Mesh<TexturedMeshVertex>>>) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::new(1.0, 0.0, 0.0, 1.0),
            mesh: Weak::from(Rc::downgrade(mesh)),
            command_buffer: vec![],
        }
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        let translation = nalgebra_glm::translation(&self.position);
        let rotation = nalgebra_glm::quat_to_mat4(&self.rotation);
        let scale = nalgebra_glm::scaling(&self.scale);
        translation * rotation * scale
    }

    pub fn update_command_buffer(
        &mut self, 
        device: &Device,
        data: &VulkanData,
        pipeline: &Pipeline,
        image_index: usize
    ) -> Result<vk::CommandBuffer>{
        if image_index >= self.command_buffer.len() {
            self.command_buffer.resize(image_index + 1, vk::CommandBuffer::null());
        }
        if self.command_buffer[image_index].is_null() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pools[image_index])
            .level(vk::CommandBufferLevel::SECONDARY)
            .command_buffer_count(1);

            self.command_buffer[image_index] = unsafe {
                device.allocate_command_buffers(&allocate_info)?[0]
            };
        }
        let command_buffer = self.command_buffer[image_index];

        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(data.render_pass)
            .subpass(0)
            .framebuffer(data.framebuffers[image_index]);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        let mesh = self.mesh.upgrade().ok_or(anyhow!("Mesh has been dropped"))?;
        let mesh = mesh.as_ref().borrow();

        unsafe{
            device.begin_command_buffer(command_buffer, &info)?;

            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_object.vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(command_buffer, mesh.vertex_object.index_buffer, 0, vk::IndexType::UINT32);
            device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline_layout, 0, &[pipeline.descriptor_sets[image_index]], &[]);
            device.cmd_draw_indexed(command_buffer, mesh.indices.len() as u32, 1, 0, 0, 0);
            device.end_command_buffer(command_buffer)?;
        }
        Ok(command_buffer)
    }
} 

pub type InstanceManager = ResourceManager<RenderInstance>;