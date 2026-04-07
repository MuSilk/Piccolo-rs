use std::path::Path;

use anyhow::Result;

use vulkanalia::{prelude::v1_0::*};

use crate::function::{render::{font_atlas::{create_ascii_font_texture_r32f, get_ascii_character_texture_rect}, interface::vulkan::vulkan_rhi::VulkanRHI, render_system::RenderSystem}};

#[derive(Default)]
pub struct DebugDrawFont{
    m_font_image: vk::Image,
    m_font_image_view: vk::ImageView,
    m_font_image_memory: vk::DeviceMemory,
}

impl DebugDrawFont {
    pub fn create(rhi: &VulkanRHI, font_path: &Path) -> Result<Self> {
        let mut font = Self::default();
        font.load_font(rhi, font_path)?;
        Ok(font)
    }

    pub fn get_character_texture_rect(character: u8) -> (f32, f32, f32, f32) { 
        get_ascii_character_texture_rect(character)
    }

    pub fn get_image_view(&self) -> vk::ImageView {
        self.m_font_image_view
    }

    pub fn destroy(&mut self, render_system: &RenderSystem){
        let rhi = render_system.get_rhi();
        rhi.borrow().free_memory(self.m_font_image_memory);
        rhi.borrow().destroy_image_view(self.m_font_image_view);
        rhi.borrow().destroy_image(self.m_font_image);
    }

    fn load_font(&mut self, rhi: &VulkanRHI, font_path: &Path) -> Result<()> {
        let (image, image_memory,image_view) = create_ascii_font_texture_r32f(rhi, font_path)?;

        self.m_font_image = image;
        self.m_font_image_memory = image_memory;
        self.m_font_image_view = image_view;

        Ok(())
    }
}


