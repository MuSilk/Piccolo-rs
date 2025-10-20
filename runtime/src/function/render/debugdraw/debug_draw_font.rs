use std::path::Path;

use ab_glyph::{Font, ScaleFont};
use anyhow::Result;

use vulkanalia::{prelude::v1_0::*};

use crate::{function::{global::global_context::RuntimeGlobalContext, render::interface::vulkan::vulkan_rhi::VulkanRHI}};

#[derive(Default)]
pub struct DebugDrawFont{
    m_font_image: vk::Image,
    m_font_image_view: vk::ImageView,
    m_font_image_memory: vk::DeviceMemory,
}

impl DebugDrawFont {
    const RANGE_L : u8 = 32;
    const RANGE_R : u8 = 126;
    const SINGLE_CHARACTER_WIDTH : i32 = 32;
    const SINGLE_CHARACTER_HEIGHT : i32 = 64;
    const NUM_OF_CHARACTER_IN_ONE_LINE : i32 = 16;
    const NUM_OF_CHARACTER: i32 = (Self::RANGE_R - Self::RANGE_L + 1) as i32;
    const BITMAP_W: i32 = Self::SINGLE_CHARACTER_WIDTH * Self::NUM_OF_CHARACTER_IN_ONE_LINE;
    const BITMAP_H: i32 = Self::SINGLE_CHARACTER_HEIGHT * ((Self::NUM_OF_CHARACTER + Self::NUM_OF_CHARACTER_IN_ONE_LINE - 1) / Self::NUM_OF_CHARACTER_IN_ONE_LINE);

    pub fn create(rhi: &VulkanRHI, font_path: &Path) -> Result<Self> {
        let mut font = Self::default();
        font.load_font(rhi, font_path)?;
        Ok(font)
    }

    pub fn get_character_texture_rect(character: u8) -> (f32, f32, f32, f32) { 
        if character >= Self::RANGE_L && character <= Self::RANGE_R {(
            ((character - Self::RANGE_L) as i32 % Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_WIDTH) as f32 / Self::BITMAP_W as f32,
            ((character - Self::RANGE_L) as i32 % Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_WIDTH + Self::SINGLE_CHARACTER_WIDTH) as f32 / Self::BITMAP_W as f32,
            ((character - Self::RANGE_L) as i32 / Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_HEIGHT) as f32 / Self::BITMAP_H as f32,
            ((character - Self::RANGE_L) as i32 / Self::NUM_OF_CHARACTER_IN_ONE_LINE * Self::SINGLE_CHARACTER_HEIGHT + Self::SINGLE_CHARACTER_HEIGHT) as f32 / Self::BITMAP_H as f32,
        )}
        else{
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn get_image_view(&self) -> vk::ImageView {
        self.m_font_image_view
    }

    pub fn destroy(&mut self){
        let render_system = RuntimeGlobalContext::get_render_system().borrow();
        let rhi = render_system.get_rhi();
        rhi.borrow().free_memory(self.m_font_image_memory);
        rhi.borrow().destroy_image_view(self.m_font_image_view);
        rhi.borrow().destroy_image(self.m_font_image);
    }

    fn load_font(&mut self, rhi: &VulkanRHI, font_path: &Path) -> Result<()> {
        let mut image_data = Vec::<f32>::new();
        image_data.resize((Self::BITMAP_W*Self::BITMAP_H) as usize, 0.0);

        let font_buffer = std::fs::read(font_path)?;
        let face = ab_glyph::FontArc::try_from_vec(font_buffer).unwrap();
        let scale = ab_glyph::PxScale::from(Self::SINGLE_CHARACTER_HEIGHT as f32 - 2.0);
        let font = face.as_scaled(scale);

        let ascent =  font.ascent();

        for charactor in Self::RANGE_L..=Self::RANGE_R { 
            let id = font.glyph_id(charactor as char);
            let left_side_bearing = font.h_side_bearing(id);

            let glyph = id.with_scale(scale);
            let glyph = match font.outline_glyph(glyph) {
                None => continue,
                Some(glyph) => glyph,
            };
            let bitmap_box = glyph.px_bounds();
            let y = ascent + bitmap_box.min.y - 2.0;

            let charactor_x =  (charactor - Self::RANGE_L) as i32 % Self::NUM_OF_CHARACTER_IN_ONE_LINE;
            let charactor_y =  (charactor - Self::RANGE_L) as i32 / Self::NUM_OF_CHARACTER_IN_ONE_LINE;

            let byte_offset = left_side_bearing + 
                (charactor_x * Self::SINGLE_CHARACTER_WIDTH) as f32 + 
                ((charactor_y * Self::SINGLE_CHARACTER_HEIGHT + y as i32) * Self::BITMAP_W) as f32;

            glyph.draw(|gx, gy, v| {
                let index = (byte_offset + gx as f32 + gy as f32 * Self::BITMAP_W as f32) as usize;
                image_data[index] = v;
            });
        }    

        let pixels = unsafe {
            std::slice::from_raw_parts(
                image_data.as_ptr() as *const u8,
                image_data.len() * std::mem::size_of::<f32>()
            )
        };

        let (image, image_memory,image_view) = rhi.create_texture_image(
            Self::BITMAP_W as u32,
            Self::BITMAP_H as u32, 
            pixels,
            vk::Format::R32_SFLOAT, 
            0)?;

        self.m_font_image = image;
        self.m_font_image_memory = image_memory;
        self.m_font_image_view = image_view;
        
        Ok(())
    }
}


