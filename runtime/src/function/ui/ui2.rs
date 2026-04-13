use anyhow::Result;
use std::path::Path;
use vulkanalia::prelude::v1_0::*;
use crate::{
    function::render::{
        font_atlas::{create_ascii_font_texture_rgba, get_ascii_character_texture_rect},
        interface::vulkan::vulkan_rhi::VulkanRHI,
    },
    resource::config_manager::ConfigManager,
};
use bitflags::bitflags;

#[derive(Clone, Debug, Default)]
pub struct UiInputSnapshot {
    pub mouse_pos: [f32; 2],
    pub mouse_down: [bool; 3],
    pub mouse_wheel: f32,
}

#[derive(Clone, Debug, Default)]
pub struct UiFrame {
    pub frame_id: u64,
    pub dt: f32,
    pub viewport: [f32; 2],
    pub input: UiInputSnapshot,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

pub enum UiDrawCmd {
    DrawIndexed {
        first_index: u32,
        index_count: u32,
        vertex_offset: i32,
        clip_rect: [f32; 4],
        texture_id: u32,
    },
}

pub const UI_TEXTURE_ID_FONT_ATLAS: u32 = 0;

#[derive(Copy, Clone, Debug, Default)]
pub struct UiTextureResource {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}

#[derive(Default)]
pub struct UiDrawList {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub commands: Vec<UiDrawCmd>,
}

#[derive(Default)]
pub struct UiRuntime {
    frame_counter: u64,
    prev_input: UiInputSnapshot,
    current_input: UiInputSnapshot,
    draw_list: UiDrawList,
    viewport: [f32; 2],
    active_id: Option<u64>,
    hover_id: Option<u64>,
    prev_hover_id: Option<u64>,
    menu_bar_active: bool,
    menu_cursor_x: f32,
    menu_popup_open: Option<String>,
    current_menu_label: Option<String>,
    current_menu_popup_pos: [f32; 2],
    current_menu_popup_size: [f32; 2],
    current_menu_item_cursor_y: f32,
    textures: Vec<Option<UiTextureResource>>,
    textures_dirty: bool,
}

pub struct UiButtonResult {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

pub struct UiPanel {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub body_pos: [f32; 2],
    pub body_size: [f32; 2],
    pub clip_rect: [f32; 4],
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct UiPanelFlags: u32 {
        const BODY_BG = 1 << 0;
        const HEADER_BG = 1 << 1;
        const BORDER = 1 << 2;
    }
}

impl Default for UiPanelFlags {
    fn default() -> Self {
        Self::BODY_BG | Self::HEADER_BG | Self::BORDER
    }
}

impl UiRuntime {
    pub fn load_texture_from_path(&mut self, rhi: &VulkanRHI, image_path: &Path) -> Result<u32> {
        let image = image::open(image_path)?;
        let image = image.to_rgba8();
        let width = image.width();
        let height = image.height();
        let pixels = image.into_raw();
        let (texture_image, texture_memory, texture_view) = rhi.create_texture_image(
            width,
            height,
            &pixels,
            vk::Format::R8G8B8A8_UNORM,
            0,
        )?;

        let texture_id = self.textures.len() as u32;
        self.set_texture(
            texture_id,
            UiTextureResource {
                image: texture_image,
                view: texture_view,
                memory: texture_memory,
            },
        );
        Ok(texture_id)
    }

    pub fn load_font_texture(
        &mut self,
        rhi: &VulkanRHI,
        config_manager: &ConfigManager,
    ) -> Result<u32> {
        if let Some(old_texture) = self.get_texture(UI_TEXTURE_ID_FONT_ATLAS) {
            if old_texture.view != vk::ImageView::null() {
                rhi.destroy_image_view(old_texture.view);
            }
            if old_texture.image != vk::Image::null() {
                rhi.destroy_image(old_texture.image);
            }
            if old_texture.memory != vk::DeviceMemory::null() {
                rhi.free_memory(old_texture.memory);
            }
        }
        let font_path = config_manager.get_editor_font_path().to_path_buf();
        let (image, memory, view) = create_ascii_font_texture_rgba(rhi, font_path.as_path())?;
        self.set_texture(
            UI_TEXTURE_ID_FONT_ATLAS,
            UiTextureResource {
                image,
                view,
                memory,
            },
        );
        Ok(UI_TEXTURE_ID_FONT_ATLAS)
    }

    pub fn destroy_textures(&mut self, rhi: &VulkanRHI) {
        for texture in self.textures.iter_mut().filter_map(Option::take) {
            if texture.view != vk::ImageView::null() {
                rhi.destroy_image_view(texture.view);
            }
            if texture.image != vk::Image::null() {
                rhi.destroy_image(texture.image);
            }
            if texture.memory != vk::DeviceMemory::null() {
                rhi.free_memory(texture.memory);
            }
        }
        self.textures_dirty = true;
    }

    fn set_texture(&mut self, texture_id: u32, texture: UiTextureResource) {
        let texture_index = texture_id as usize;
        if self.textures.len() <= texture_index {
            self.textures.resize(texture_index + 1, None);
        }

        if let Some(old_texture) = self.textures[texture_index].replace(texture) {
            let _ = old_texture; // Caller owns replacement lifecycle.
        }
        self.textures_dirty = true;
    }

    pub fn get_texture(&self, texture_id: u32) -> Option<UiTextureResource> {
        self.textures
            .get(texture_id as usize)
            .and_then(|texture| *texture)
    }

    pub fn has_texture(&self, texture_id: u32) -> bool {
        self.get_texture(texture_id).is_some()
    }

    pub fn update_input(&mut self, input: UiInputSnapshot) {
        self.prev_input = self.current_input.clone();
        self.current_input = input;
    }

    pub fn set_viewport(&mut self, viewport: [f32; 2]) {
        self.viewport = viewport;
    }

    pub fn new_frame(&mut self) {
        self.frame_counter += 1;
        self.draw_list = UiDrawList::default();
        self.prev_hover_id = self.hover_id;
        self.hover_id = None;
    }

    pub fn get_viewport(&self) -> [f32; 2] {
        self.viewport
    }

    pub fn build_frame(&self, dt: f32) -> (UiFrame, &UiDrawList) {
        let frame = UiFrame {
            frame_id: self.frame_counter,
            dt,
            viewport: self.viewport,
            input: self.current_input.clone(),
        };
        (frame, &self.draw_list)
    }

    pub fn mouse_pos(&self) -> [f32; 2] {
        self.current_input.mouse_pos
    }

    pub fn mouse_down(&self, button_index: usize) -> bool {
        self.current_input
            .mouse_down
            .get(button_index)
            .copied()
            .unwrap_or(false)
    }

    pub fn mouse_pressed(&self, button_index: usize) -> bool {
        let curr = self
            .current_input
            .mouse_down
            .get(button_index)
            .copied()
            .unwrap_or(false);
        let prev = self
            .prev_input
            .mouse_down
            .get(button_index)
            .copied()
            .unwrap_or(false);
        curr && !prev
    }

    pub fn mouse_released(&self, button_index: usize) -> bool {
        let curr = self
            .current_input
            .mouse_down
            .get(button_index)
            .copied()
            .unwrap_or(false);
        let prev = self
            .prev_input
            .mouse_down
            .get(button_index)
            .copied()
            .unwrap_or(false);
        !curr && prev
    }

    pub fn push_colored_rect(
        &mut self, pos: [f32; 2], size: [f32; 2], color: [u8; 4], clip_rect: [f32; 4]
    ) {
        push_colored_rect(
            &mut self.draw_list,
            pos,
            size,
            color,
            clip_rect,
        );
    }

    pub fn push_textured_rect(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv_rect: [f32; 4],
        color: [u8; 4],
        clip_rect: [f32; 4],
        texture_id: u32,
    ) {
        push_textured_rect(
            &mut self.draw_list,
            pos,
            size,
            uv_rect,
            color,
            clip_rect,
            texture_id,
        );
    }

    pub fn push_text_ascii(
        &mut self, text: &str, pos: [f32; 2], glyph_size: [f32; 2], color: [u8; 4], clip_rect: [f32; 4]
    ) {
        push_text_ascii(
            &mut self.draw_list,
            text,
            pos,
            glyph_size,
            color,
            clip_rect,
        )
    }

    pub fn button(
        &mut self,
        id: &str,
        text: &str,
        pos: [f32; 2],
        size: [f32; 2],
    ) -> UiButtonResult {
        let clip = [0.0, 0.0, self.viewport[0], self.viewport[1]];
        self.button_with_clip(id, text, pos, size, clip)
    }

    pub fn button_in_clip(
        &mut self,
        id: &str,
        text: &str,
        pos: [f32; 2],
        size: [f32; 2],
        clip_rect: [f32; 4],
    ) -> UiButtonResult {
        self.button_with_clip(id, text, pos, size, clip_rect)
    }

    pub fn panel(
        &mut self,
        id: &str,
        title: &str,
        pos: [f32; 2],
        size: [f32; 2],
        flags: UiPanelFlags,
    ) -> UiPanel {
        let clip = [0.0, 0.0, self.viewport[0], self.viewport[1]];
        let header_h = 24.0;
        let bg = [36, 40, 52, 235];
        let header_bg = [54, 60, 78, 245];
        let border = [110, 120, 150, 200];

        if flags.contains(UiPanelFlags::BODY_BG) {
            push_colored_rect(&mut self.draw_list, pos, size, bg, clip);
        }
        if flags.contains(UiPanelFlags::HEADER_BG) {
            push_colored_rect(
                &mut self.draw_list,
                [pos[0], pos[1]],
                [size[0], header_h],
                header_bg,
                clip,
            );
        }
        if flags.contains(UiPanelFlags::BORDER) {
            push_rect_border(&mut self.draw_list, pos, size, 1.0, border, clip);
        }
        push_text_ascii(
            &mut self.draw_list,
            title,
            [pos[0] + 8.0, pos[1] + 5.0],
            [8.0, 14.0],
            [235, 240, 250, 255],
            clip,
        );

        let _ = id;
        UiPanel {
            pos,
            size,
            body_pos: [pos[0] + 4.0, pos[1] + header_h + 4.0],
            body_size: [size[0] - 8.0, (size[1] - header_h - 8.0).max(0.0)],
            clip_rect: [pos[0], pos[1] + header_h, pos[0] + size[0], pos[1] + size[1]],
        }
    }

    fn button_with_clip(
        &mut self,
        id: &str,
        text: &str,
        pos: [f32; 2],
        size: [f32; 2],
        clip: [f32; 4],
    ) -> UiButtonResult {
        let widget_id = hash_widget_id(id);
        let mouse = self.current_input.mouse_pos;
        let hovered = point_in_rect(mouse, pos, size);
        if hovered {
            // Last hit in frame wins, matching top-most draw order.
            self.hover_id = Some(widget_id);
        }
        let was_down = self.prev_input.mouse_down[0];
        let is_down = self.current_input.mouse_down[0];
        let just_pressed = !was_down && is_down;
        let just_released = was_down && !is_down;

        if hovered && just_pressed {
            self.active_id = Some(widget_id);
        }
        let pressed = self.active_id == Some(widget_id) && is_down;
        let clicked = self.active_id == Some(widget_id) && hovered && just_released;
        if self.active_id == Some(widget_id) && just_released {
            self.active_id = None;
        }

        // Use previous frame hover id for stable overlap highlight.
        let hover_for_render = hovered && self.prev_hover_id == Some(widget_id);
        let bg = if pressed {
            [70, 130, 240, 255]
        } else if hover_for_render {
            [85, 145, 255, 255]
        } else {
            [50, 100, 200, 255]
        };
        push_colored_rect(&mut self.draw_list, pos, size, bg, clip);
        // Keep button label inside bounds and vertically centered.
        let glyph = [8.0, 14.0];
        let text_x = pos[0] + 8.0;
        let text_y = pos[1] + ((size[1] - glyph[1]) * 0.5).max(0.0);
        push_text_ascii(
            &mut self.draw_list,
            text,
            [text_x, text_y],
            glyph,
            [255, 255, 255, 255],
            clip,
        );

        UiButtonResult {
            hovered,
            pressed,
            clicked,
        }
    }

    pub fn begin_main_menu_bar(&mut self) -> bool {
        let clip = [0.0, 0.0, self.viewport[0], self.viewport[1]];
        push_colored_rect(
            &mut self.draw_list,
            [0.0, 0.0],
            [self.viewport[0], 30.0],
            [28, 32, 42, 240],
            clip,
        );
        self.menu_bar_active = true;
        self.menu_cursor_x = 8.0;
        self.current_menu_label = None;
        true
    }

    pub fn begin_menu(&mut self, label: &str) -> bool {
        if !self.menu_bar_active {
            return false;
        }
        let label_width = 10.0 * label.len() as f32;
        let width = 16.0 + label_width;
        let header_pos = [self.menu_cursor_x, 3.0];
        let resp = self.button(
            &format!("MenuHeader::{label}"),
            label,
            header_pos,
            [width, 24.0],
        );
        if resp.clicked {
            match self.menu_popup_open.as_ref() {
                Some(open) if open == label => self.menu_popup_open = None,
                _ => self.menu_popup_open = Some(label.to_string()),
            }
        }

        let is_open = self
            .menu_popup_open
            .as_ref()
            .map(|x| x == label)
            .unwrap_or(false);
        if is_open {
            let popup_pos = [header_pos[0], 30.0];
            let popup_size = [220.0, 220.0];
            let clip = [0.0, 0.0, self.viewport[0], self.viewport[1]];
            push_colored_rect(
                &mut self.draw_list,
                popup_pos,
                popup_size,
                [36, 40, 52, 248],
                clip,
            );
            self.current_menu_label = Some(label.to_string());
            self.current_menu_popup_pos = popup_pos;
            self.current_menu_popup_size = popup_size;
            self.current_menu_item_cursor_y = popup_pos[1] + 6.0;
        }
        self.menu_cursor_x += width + 8.0;
        is_open
    }

    pub fn menu_item(&mut self, label: &str) -> bool {
        if !self.menu_bar_active || self.current_menu_label.is_none() {
            return false;
        }
        let width = self.current_menu_popup_size[0] - 12.0;
        let pos = [self.current_menu_popup_pos[0] + 6.0, self.current_menu_item_cursor_y];
        let clicked = self
            .button(
                &format!(
                    "MenuItem::{}::{label}",
                    self.current_menu_label.as_ref().unwrap()
                ),
                label,
                pos,
                [width, 24.0],
            )
            .clicked;
        self.current_menu_item_cursor_y += 26.0;
        if clicked {
            self.menu_popup_open = None;
        }
        clicked
    }

    pub fn menu_item_config<'a>(&'a mut self, label: &str) -> UiMenuItemConfig<'a> {
        UiMenuItemConfig {
            ui: self,
            label: label.to_string(),
        }
    }

    pub fn end_main_menu_bar(&mut self) {
        self.menu_bar_active = false;
        self.current_menu_label = None;
    }
}

pub struct UiMenuItemConfig<'a> {
    ui: &'a mut UiRuntime,
    label: String,
}

impl<'a> UiMenuItemConfig<'a> {
    pub fn build_with_ref(self, value: &mut bool) -> bool {
        let text = if *value {
            format!("{}:On", self.label)
        } else {
            format!("{}:Off", self.label)
        };
        let clicked = self.ui.menu_item(&text);
        if clicked {
            *value = !*value;
        }
        clicked
    }
}

fn point_in_rect(p: [f32; 2], pos: [f32; 2], size: [f32; 2]) -> bool {
    p[0] >= pos[0] && p[0] <= pos[0] + size[0] && p[1] >= pos[1] && p[1] <= pos[1] + size[1]
}

fn hash_widget_id(id: &str) -> u64 {
    let mut h = 1469598103934665603_u64;
    for b in id.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211_u64);
    }
    h
}

fn push_colored_rect(
    draw_list: &mut UiDrawList,
    pos: [f32; 2],
    size: [f32; 2],
    color: [u8; 4],
    clip_rect: [f32; 4],
) {
    push_textured_rect(
        draw_list,
        pos,
        size,
        [0.0, 0.0, 0.0, 0.0],
        color,
        clip_rect,
        UI_TEXTURE_ID_FONT_ATLAS,
    );
}

fn push_textured_rect(
    draw_list: &mut UiDrawList,
    pos: [f32; 2],
    size: [f32; 2],
    uv_rect: [f32; 4],
    color: [u8; 4],
    clip_rect: [f32; 4],
    texture_id: u32,
) {
    let base = draw_list.vertices.len() as u32;
    let x = pos[0];
    let y = pos[1];
    let w = size[0];
    let h = size[1];
    let u0 = uv_rect[0];
    let v0 = uv_rect[1];
    let u1 = uv_rect[2];
    let v1 = uv_rect[3];

    // Colored rects can still use a fixed white pixel UV.
    // Texture-capable widgets should provide their own UVs and texture_id.
    draw_list.vertices.extend_from_slice(&[
        UiVertex { pos: [x, y], uv: [u0, v0], col: color },
        UiVertex { pos: [x + w, y], uv: [u1, v0], col: color },
        UiVertex { pos: [x + w, y + h], uv: [u1, v1], col: color },
        UiVertex { pos: [x, y + h], uv: [u0, v1], col: color },
    ]);
    draw_list
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    draw_list.commands.push(UiDrawCmd::DrawIndexed {
        first_index: draw_list.indices.len() as u32 - 6,
        index_count: 6,
        vertex_offset: 0,
        clip_rect,
        texture_id,
    });
}

fn push_rect_border(
    draw_list: &mut UiDrawList,
    pos: [f32; 2],
    size: [f32; 2],
    thickness: f32,
    color: [u8; 4],
    clip_rect: [f32; 4],
) {
    if thickness <= 0.0 || size[0] <= 0.0 || size[1] <= 0.0 {
        return;
    }
    // top
    push_colored_rect(draw_list, [pos[0], pos[1]], [size[0], thickness], color, clip_rect);
    // bottom
    push_colored_rect(
        draw_list,
        [pos[0], pos[1] + size[1] - thickness],
        [size[0], thickness],
        color,
        clip_rect,
    );
    // left
    push_colored_rect(draw_list, [pos[0], pos[1]], [thickness, size[1]], color, clip_rect);
    // right
    push_colored_rect(
        draw_list,
        [pos[0] + size[0] - thickness, pos[1]],
        [thickness, size[1]],
        color,
        clip_rect,
    );
}

fn push_text_ascii(
    draw_list: &mut UiDrawList,
    text: &str,
    pos: [f32; 2],
    glyph_size: [f32; 2],
    color: [u8; 4],
    clip_rect: [f32; 4],
) {
    let start_index = draw_list.indices.len() as u32;
    let mut pen_x = pos[0];
    let pen_y = pos[1];
    let w = glyph_size[0];
    let h = glyph_size[1];

    for ch in text.bytes() {
        if ch == b' ' {
            pen_x += w;
            continue;
        }

        let (u0, u1, v0, v1) = get_ascii_character_texture_rect(ch);
        if u0 == 0.0 && u1 == 0.0 && v0 == 0.0 && v1 == 0.0 {
            pen_x += w;
            continue;
        }

        let base = draw_list.vertices.len() as u32;
        draw_list.vertices.extend_from_slice(&[
            UiVertex {
                pos: [pen_x, pen_y],
                uv: [u0, v0],
                col: color,
            },
            UiVertex {
                pos: [pen_x + w, pen_y],
                uv: [u1, v0],
                col: color,
            },
            UiVertex {
                pos: [pen_x + w, pen_y + h],
                uv: [u1, v1],
                col: color,
            },
            UiVertex {
                pos: [pen_x, pen_y + h],
                uv: [u0, v1],
                col: color,
            },
        ]);
        draw_list
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        pen_x += w;
    }

    let index_count = draw_list.indices.len() as u32 - start_index;
    if index_count > 0 {
        draw_list.commands.push(UiDrawCmd::DrawIndexed {
            first_index: start_index,
            index_count,
            vertex_offset: 0,
            clip_rect,
            texture_id: UI_TEXTURE_ID_FONT_ATLAS,
        });
    }
}
