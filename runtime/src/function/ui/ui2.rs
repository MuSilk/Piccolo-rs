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
    current_input: UiInputSnapshot,
}

impl UiRuntime {
    pub fn update_input(&mut self, input: UiInputSnapshot) {
        self.current_input = input;
    }

    pub fn build_frame(&mut self, dt: f32, viewport: [f32; 2]) -> (UiFrame, UiDrawList) {
        self.frame_counter += 1;
        let frame = UiFrame {
            frame_id: self.frame_counter,
            dt,
            viewport,
            input: self.current_input.clone(),
        };

        // Phase-1 skeleton: emit one demo panel for end-to-end validation.
        let mut draw_list = UiDrawList::default();
        push_colored_rect(
            &mut draw_list,
            [viewport[0] * 0.25, viewport[1] * 0.25],
            [viewport[0] * 0.25, viewport[1] * 0.25],
            [60, 120, 220, 255],
            [0.0, 0.0, viewport[0], viewport[1]],
        );
        push_text_ascii(
            &mut draw_list,
            "UI2 TEXT TEST 0123",
            [viewport[0] * 0.28, viewport[1] * 0.33],
            [14.0, 28.0],
            [255, 255, 255, 255],
            [0.0, 0.0, viewport[0], viewport[1]],
        );
        (frame, draw_list)
    }
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
