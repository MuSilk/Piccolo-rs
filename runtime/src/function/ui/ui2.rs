use crate::function::render::font_atlas::get_ascii_character_texture_rect;

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
}

pub struct UiButtonResult {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

impl UiRuntime {
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
    }

    pub fn get_viewport(&self) -> [f32; 2] {
        self.viewport
    }

    pub fn build_frame(&mut self, dt: f32) -> (UiFrame, &UiDrawList) {
        let frame = UiFrame {
            frame_id: self.frame_counter,
            dt,
            viewport: self.viewport,
            input: self.current_input.clone(),
        };
        (frame, &self.draw_list)
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

        let widget_id = hash_widget_id(id);
        let mouse = self.current_input.mouse_pos;
        let hovered = point_in_rect(mouse, pos, size);
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

        let bg = if pressed {
            [70, 130, 240, 255]
        } else if hovered {
            [85, 145, 255, 255]
        } else {
            [50, 100, 200, 255]
        };
        let clip = [0.0, 0.0, self.viewport[0], self.viewport[1]];
        push_colored_rect(&mut self.draw_list, pos, size, bg, clip);
        push_text_ascii(
            &mut self.draw_list,
            text,
            [pos[0] + 12.0, pos[1] + 10.0],
            [12.0, 24.0],
            [255, 255, 255, 255],
            clip,
        );

        UiButtonResult {
            hovered,
            pressed,
            clicked,
        }
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
    let base = draw_list.vertices.len() as u32;
    let x = pos[0];
    let y = pos[1];
    let w = size[0];
    let h = size[1];

    // Current UI shader multiplies vertex color by sampled font texture.
    // Keep UV fixed at atlas white-pixel to render a solid colored quad.
    draw_list.vertices.extend_from_slice(&[
        UiVertex { pos: [x, y], uv: [0.0, 0.0], col: color },
        UiVertex { pos: [x + w, y], uv: [0.0, 0.0], col: color },
        UiVertex { pos: [x + w, y + h], uv: [0.0, 0.0], col: color },
        UiVertex { pos: [x, y + h], uv: [0.0, 0.0], col: color },
    ]);
    draw_list
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    draw_list.commands.push(UiDrawCmd::DrawIndexed {
        first_index: draw_list.indices.len() as u32 - 6,
        index_count: 6,
        vertex_offset: 0,
        clip_rect,
        texture_id: 0,
    });
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
            texture_id: 0,
        });
    }
}
