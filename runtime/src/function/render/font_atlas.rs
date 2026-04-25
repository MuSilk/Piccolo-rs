use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

#[derive(Clone, Debug)]
pub struct FontAtlasGlyph {
    pub x0_px: f32,
    pub x1_px: f32,
    pub y0_px: f32,
    pub y1_px: f32,
    pub advance_px: f32,
    pub side_bearing_px: f32,
}

pub struct FontAtlas {
    font_path: PathBuf,
    glyph_width: i32,
    glyph_height: i32,
    glyph_per_line: i32,
    bitmap_w: i32,
    bitmap_h: i32,
    glyph_map: HashMap<char, FontAtlasGlyph>,
    data: Vec<f32>,
    next_slot: i32,
    font: Option<ab_glyph::FontArc>,
    is_dirty: bool,
}

impl FontAtlas {
    pub fn new(font_path: &Path, glyph_width: i32, glyph_height: i32) -> Self {
        let glyph_per_line = 16;
        let bitmap_w = glyph_width * glyph_per_line;
        let bitmap_h = glyph_height.max(1);
        Self {
            font_path: font_path.to_path_buf(),
            glyph_width,
            glyph_height,
            glyph_per_line,
            bitmap_w,
            bitmap_h,
            glyph_map: HashMap::new(),
            data: vec![0.0; (bitmap_w * bitmap_h) as usize],
            next_slot: 0,
            font: None,
            is_dirty: false,
        }
    }

    fn get_or_load_font(&mut self) -> Result<ab_glyph::FontArc> {
        if let Some(font) = &self.font {
            return Ok(font.clone());
        }
        let font_buffer = std::fs::read(&self.font_path)?;
        let font = ab_glyph::FontArc::try_from_vec(font_buffer).map_err(|e| {
            anyhow::anyhow!("Failed to load font '{}': {e}", self.font_path.display())
        })?;
        self.font = Some(font.clone());
        Ok(font)
    }

    fn ensure_capacity_for_slot(&mut self, slot: i32) {
        let row = slot / self.glyph_per_line;
        let required_h = (row + 1) * self.glyph_height;
        if required_h <= self.bitmap_h {
            return;
        }
        self.bitmap_h = required_h;
        self.data
            .resize((self.bitmap_w * self.bitmap_h) as usize, 0.0);
    }

    pub fn get_character_texture_rect(&mut self, character: char) -> Result<FontAtlasGlyph> {
        if !self.glyph_map.contains_key(&character) {
            let face = self.get_or_load_font()?;
            let scale = ab_glyph::PxScale::from(self.glyph_height as f32);
            let font = face.as_scaled(scale);
            let ascent = font.ascent();
            let id = font.glyph_id(character);
            let left_side_bearing = font.h_side_bearing(id);
            let glyph = id.with_scale(scale);
            let glyph = match font.outline_glyph(glyph) {
                None => {
                    return Err(anyhow::anyhow!(
                        "Failed to get character glyph: {}",
                        character
                    ));
                }
                Some(glyph) => glyph,
            };
            let bitmap_box = glyph.px_bounds();
            let baseline_y = (ascent + bitmap_box.min.y).floor() as i32;

            let slot = self.next_slot;
            self.next_slot += 1;
            self.ensure_capacity_for_slot(slot);

            let col = slot % self.glyph_per_line;
            let row = slot / self.glyph_per_line;
            let cell_x = col * self.glyph_width;
            let cell_y = row * self.glyph_height;
            let glyph_origin_x = cell_x + left_side_bearing.floor() as i32;
            let glyph_origin_y = cell_y + baseline_y;
            let advance_px = font.h_advance(id).floor() as f32;
            let side_bearing_px = font.h_side_bearing(id).floor() as f32;

            glyph.draw(|gx, gy, v| {
                let px = glyph_origin_x + gx as i32;
                let py = glyph_origin_y + gy as i32;
                if px < 0 || py < 0 || px >= self.bitmap_w || py >= self.bitmap_h {
                    return;
                }
                let index = (py * self.bitmap_w + px) as usize;
                self.data[index] = v;
            });
            self.glyph_map.insert(
                character,
                FontAtlasGlyph {
                    x0_px: cell_x as f32,
                    x1_px: (cell_x + self.glyph_width) as f32,
                    y0_px: cell_y as f32,
                    y1_px: (cell_y + self.glyph_height) as f32,
                    advance_px,
                    side_bearing_px,
                },
            );
            self.is_dirty = true;
        }
        Ok(self.glyph_map.get(&character).unwrap().clone())
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn bitmap_size(&self) -> (i32, i32) {
        (self.bitmap_w, self.bitmap_h)
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }
}
