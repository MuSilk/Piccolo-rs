use std::path::Path;

use ab_glyph::{Font, ScaleFont};
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;

pub const ASCII_RANGE_L: u8 = 31;
pub const ASCII_RANGE_R: u8 = 126;
pub const ASCII_GLYPH_WIDTH: i32 = 32;
pub const ASCII_GLYPH_HEIGHT: i32 = 64;
pub const ASCII_GLYPH_PER_LINE: i32 = 16;
pub const ASCII_GLYPH_COUNT: i32 = (ASCII_RANGE_R - ASCII_RANGE_L + 1) as i32;
pub const ASCII_BITMAP_W: i32 = ASCII_GLYPH_WIDTH * ASCII_GLYPH_PER_LINE;
pub const ASCII_BITMAP_H: i32 =
    ASCII_GLYPH_HEIGHT * ((ASCII_GLYPH_COUNT + ASCII_GLYPH_PER_LINE - 1) / ASCII_GLYPH_PER_LINE);

pub fn get_ascii_character_texture_rect(character: u8) -> (f32, f32, f32, f32) {
    if character >= ASCII_RANGE_L && character <= ASCII_RANGE_R {
        (
            ((character - ASCII_RANGE_L) as i32 % ASCII_GLYPH_PER_LINE * ASCII_GLYPH_WIDTH) as f32
                / ASCII_BITMAP_W as f32,
            ((character - ASCII_RANGE_L) as i32 % ASCII_GLYPH_PER_LINE * ASCII_GLYPH_WIDTH
                + ASCII_GLYPH_WIDTH) as f32
                / ASCII_BITMAP_W as f32,
            ((character - ASCII_RANGE_L) as i32 / ASCII_GLYPH_PER_LINE * ASCII_GLYPH_HEIGHT) as f32
                / ASCII_BITMAP_H as f32,
            ((character - ASCII_RANGE_L) as i32 / ASCII_GLYPH_PER_LINE * ASCII_GLYPH_HEIGHT
                + ASCII_GLYPH_HEIGHT) as f32
                / ASCII_BITMAP_H as f32,
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

pub fn rasterize_ascii_coverage(font_path: &Path) -> Result<Vec<f32>> {
    let mut image_data = vec![0.0_f32; (ASCII_BITMAP_W * ASCII_BITMAP_H) as usize];
    let font_buffer = std::fs::read(font_path)?;
    let face = ab_glyph::FontArc::try_from_vec(font_buffer).unwrap();
    let scale = ab_glyph::PxScale::from(ASCII_GLYPH_HEIGHT as f32 - 2.0);
    let font = face.as_scaled(scale);
    let ascent = font.ascent();

    for character in ASCII_RANGE_L..=ASCII_RANGE_R {
        let id = font.glyph_id(character as char);
        let left_side_bearing = font.h_side_bearing(id);

        let glyph = id.with_scale(scale);
        let glyph = match font.outline_glyph(glyph) {
            None => continue,
            Some(glyph) => glyph,
        };
        let bitmap_box = glyph.px_bounds();
        let y = ascent + bitmap_box.min.y - 2.0;

        let character_x = (character - ASCII_RANGE_L) as i32 % ASCII_GLYPH_PER_LINE;
        let character_y = (character - ASCII_RANGE_L) as i32 / ASCII_GLYPH_PER_LINE;

        let byte_offset = left_side_bearing
            + (character_x * ASCII_GLYPH_WIDTH) as f32
            + ((character_y * ASCII_GLYPH_HEIGHT + y as i32) * ASCII_BITMAP_W) as f32;

        glyph.draw(|gx, gy, v| {
            let index = (byte_offset + gx as f32 + gy as f32 * ASCII_BITMAP_W as f32) as usize;
            image_data[index] = v;
        });
    }

    Ok(image_data)
}

pub fn create_ascii_font_texture_r32f(
    rhi: &VulkanRHI,
    font_path: &Path,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
    let image_data = rasterize_ascii_coverage(font_path)?;
    let pixels = unsafe {
        std::slice::from_raw_parts(
            image_data.as_ptr() as *const u8,
            image_data.len() * std::mem::size_of::<f32>(),
        )
    };
    rhi.create_texture_image(
        ASCII_BITMAP_W as u32,
        ASCII_BITMAP_H as u32,
        pixels,
        vk::Format::R32_SFLOAT,
        0,
    )
}

pub fn create_ascii_font_texture_rgba(
    rhi: &VulkanRHI,
    font_path: &Path,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
    let image_data = rasterize_ascii_coverage(font_path)?;
    let mut rgba = vec![0_u8; image_data.len() * 4];
    for (i, v) in image_data.iter().enumerate() {
        let alpha = (v.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[i * 4] = 255;
        rgba[i * 4 + 1] = 255;
        rgba[i * 4 + 2] = 255;
        rgba[i * 4 + 3] = alpha;
    }
    // Reserve a guaranteed white texel at (0,0) for solid-color quads
    // that multiply vertex color by sampled texture color (e.g. ui2 debug rect).
    if !rgba.is_empty() {
        rgba[0] = 255;
        rgba[1] = 255;
        rgba[2] = 255;
        rgba[3] = 255;
    }
    rhi.create_texture_image(
        ASCII_BITMAP_W as u32,
        ASCII_BITMAP_H as u32,
        &rgba,
        vk::Format::R8G8B8A8_UNORM,
        0,
    )
}
