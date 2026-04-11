use runtime::function::ui::ui2::{UiPanel, UiRuntime};

use super::graph_io::{
    DEFAULT_GRAPH_PATH, MAIN_CAMERA_PASS_GRAPH_PATH, load_graph_from_file, save_graph_to_file,
};
use super::graph_state::RenderGraphState;
use super::graph_types::{NodeKind, input_ports, output_ports};

pub struct RenderGraphUI {
    state: RenderGraphState,
    status_text: String,
}

impl Default for RenderGraphUI {
    fn default() -> Self {
        Self {
            state: RenderGraphState::with_default_graph(),
            status_text: "Ready".to_string(),
        }
    }
}

impl RenderGraphUI {
    pub fn draw(&mut self, ui_runtime: &mut UiRuntime, panel: &UiPanel) {
        ui_runtime.push_colored_rect(panel.body_pos, panel.body_size, [22, 25, 34, 220], panel.clip_rect);
        let title = format!("Nodes:{}  Edges:{}", self.state.graph.nodes.len(), self.state.graph.edges.len());
        ui_runtime.push_text_ascii(&title, [panel.body_pos[0] + 8.0, panel.body_pos[1] + 6.0], [8.0, 14.0], [200, 210, 230, 255], panel.clip_rect);

        let toolbar_y = panel.body_pos[1] + 24.0;
        self.draw_toolbar(ui_runtime, panel, toolbar_y);

        let origin = [panel.body_pos[0], panel.body_pos[1] + 62.0];
        let mouse = ui_runtime.mouse_pos();

        let clicked_edge_idx = self.draw_edges(ui_runtime, panel, origin);
        self.draw_pending_link(ui_runtime, panel, origin, mouse);
        if ui_runtime.mouse_released(0) {
            self.state.dragging_node = None;
        }

        let mut clicked_node = None;
        let mut clicked_output: Option<(u64, String)> = None;
        let mut clicked_input: Option<(u64, String)> = None;

        for node in &self.state.graph.nodes {
            let node_size = node_box_size(&node.kind);
            let pos = [origin[0] + node.position[0], origin[1] + node.position[1]];
            let selected = self.state.selected_node == Some(node.id);
            ui_runtime.push_colored_rect(pos, node_size, [48, 58, 78, 240], panel.clip_rect);
            ui_runtime.push_colored_rect([pos[0], pos[1]], [node_size[0], 28.0], [62, 76, 104, 250], panel.clip_rect);

            let title_btn = ui_runtime.button_in_clip(
                &format!("RenderGraphNodeTitle::{}", node.id),
                &trim_label(&node.name, 20),
                [pos[0] + 6.0, pos[1] + 2.0],
                [node_size[0] - 12.0, 24.0],
                panel.clip_rect,
            );
            if title_btn.clicked {
                clicked_node = Some(node.id);
            }
            if title_btn.pressed && self.state.dragging_node.is_none() && ui_runtime.mouse_pressed(0) {
                self.state.dragging_node = Some(node.id);
                self.state.drag_offset = [mouse[0] - pos[0], mouse[1] - pos[1]];
            }

            ui_runtime.push_text_ascii(
                node_kind_name(&node.kind),
                [pos[0] + 8.0, pos[1] + 34.0],
                [8.0, 14.0],
                [190, 205, 235, 255],
                panel.clip_rect,
            );

            for (i, p) in input_ports(&node.kind).iter().enumerate() {
                let py = pos[1] + 52.0 + i as f32 * 18.0;
                let resp = ui_runtime.button_in_clip(
                    &format!("InPort::{}::{}", node.id, p.name),
                    p.name,
                    [pos[0] + 4.0, py],
                    [98.0, 16.0],
                    panel.clip_rect,
                );
                if resp.clicked {
                    clicked_input = Some((node.id, p.name.to_string()));
                }
            }
            for (i, p) in output_ports(&node.kind).iter().enumerate() {
                let py = pos[1] + 52.0 + i as f32 * 18.0;
                let resp = ui_runtime.button_in_clip(
                    &format!("OutPort::{}::{}", node.id, p.name),
                    p.name,
                    [pos[0] + node_size[0] - 102.0, py],
                    [98.0, 16.0],
                    panel.clip_rect,
                );
                if resp.clicked {
                    clicked_output = Some((node.id, p.name.to_string()));
                }
            }

            if selected {
                draw_node_highlight(ui_runtime, pos, node_size, panel.clip_rect);
            }
        }

        if let Some(idx) = clicked_edge_idx {
            self.state.select_edge_index(idx);
        }
        if let Some(id) = clicked_node {
            self.state.select_node(id);
        }
        if let Some((node, port)) = clicked_output {
            match self.state.begin_link_from_output(node, &port) {
                Ok(()) => self.status_text = format!("Linking from {node}.{port}"),
                Err(e) => self.status_text = format!("Link failed: {e}"),
            }
        }
        if let Some((node, port)) = clicked_input {
            if self.state.pending_link.is_some() {
                match self.state.link_pending_to_input(node, &port) {
                    Ok(()) => self.status_text = format!("Linked to {node}.{port}"),
                    Err(e) => self.status_text = format!("Link failed: {e}"),
                }
            } else {
                self.status_text = "Pick an output port first".to_string();
            }
        }

        if let Some(id) = self.state.dragging_node {
            let nx = (mouse[0] - origin[0] - self.state.drag_offset[0]).max(0.0);
            let ny = (mouse[1] - origin[1] - self.state.drag_offset[1]).max(0.0);
            if let Some(node) = self.state.get_node_mut(id) {
                node.position = [nx, ny];
            }
        }

        self.draw_footer(ui_runtime, panel, toolbar_y);
    }

    fn draw_toolbar(&mut self, ui_runtime: &mut UiRuntime, panel: &UiPanel, toolbar_y: f32) {
        if ui_runtime.button_in_clip("RenderGraph::AddNode", "Add Node", [panel.body_pos[0] + 8.0, toolbar_y], [120.0, 30.0], panel.clip_rect).clicked {
            self.state.add_node();
        }
        if ui_runtime.button_in_clip("RenderGraph::DeleteNode", "Delete Selected", [panel.body_pos[0] + 136.0, toolbar_y], [180.0, 30.0], panel.clip_rect).clicked {
            self.state.delete_selected_node();
            self.status_text = "Deleted selected node".to_string();
        }
        if ui_runtime.button_in_clip("RenderGraph::DeleteEdge", "Delete Edge", [panel.body_pos[0] + 324.0, toolbar_y], [140.0, 30.0], panel.clip_rect).clicked {
            self.state.delete_selected_edge();
            self.status_text = "Deleted selected edge".to_string();
        }
        if ui_runtime.button_in_clip("RenderGraph::Save", "Save", [panel.body_pos[0] + 472.0, toolbar_y], [90.0, 30.0], panel.clip_rect).clicked {
            match save_graph_to_file(&self.state.graph, DEFAULT_GRAPH_PATH) {
                Ok(()) => self.status_text = format!("Saved: {DEFAULT_GRAPH_PATH}"),
                Err(e) => self.status_text = format!("Save failed: {e}"),
            }
        }
        if ui_runtime.button_in_clip("RenderGraph::Load", "Load", [panel.body_pos[0] + 570.0, toolbar_y], [90.0, 30.0], panel.clip_rect).clicked {
            match load_graph_from_file(DEFAULT_GRAPH_PATH) {
                Ok(graph) => {
                    self.state.replace_graph(graph);
                    println!("[RenderGraph] Loaded: {DEFAULT_GRAPH_PATH}");
                }
                Err(e) => println!("[RenderGraph] Load failed ({DEFAULT_GRAPH_PATH}): {e}"),
            }
        }
        if ui_runtime.button_in_clip("RenderGraph::LoadMainPass", "Load MainPass", [panel.body_pos[0] + 668.0, toolbar_y], [150.0, 30.0], panel.clip_rect).clicked {
            match load_graph_from_file(MAIN_CAMERA_PASS_GRAPH_PATH) {
                Ok(graph) => {
                    self.state.replace_graph(graph);
                    println!("[RenderGraph] Loaded: {MAIN_CAMERA_PASS_GRAPH_PATH}");
                }
                Err(e) => {
                    println!(
                        "[RenderGraph] Load failed ({}): {e}",
                        MAIN_CAMERA_PASS_GRAPH_PATH
                    );
                }
            }
        }
        if ui_runtime.button_in_clip("RenderGraph::CancelLink", "Cancel Link", [panel.body_pos[0] + 826.0, toolbar_y], [140.0, 30.0], panel.clip_rect).clicked {
            self.state.cancel_link();
            self.status_text = "Link cancelled".to_string();
        }
        if ui_runtime.button_in_clip("RenderGraph::CycleKind", "Cycle Kind", [panel.body_pos[0] + 974.0, toolbar_y], [130.0, 30.0], panel.clip_rect).clicked {
            self.state.cycle_selected_node_kind();
            self.status_text = "Switched selected node kind".to_string();
        }
    }

    fn draw_edges(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel, origin: [f32; 2]) -> Option<usize> {
        let mut best_pick: Option<(usize, f32)> = None;
        let mouse = ui_runtime.mouse_pos();
        let just_pressed = ui_runtime.mouse_pressed(0);
        for (idx, edge) in self.state.graph.edges.iter().enumerate() {
            let Some(from_node) = self.state.get_node(edge.from_node) else {
                continue;
            };
            let Some(to_node) = self.state.get_node(edge.to_node) else {
                continue;
            };
            let from_size = node_box_size(&from_node.kind);
            let Some(from_pos) = output_port_anchor(origin, from_size, from_node.position, &from_node.kind, &edge.from_port) else {
                continue;
            };
            let Some(to_pos) = input_port_anchor(origin, to_node.position, &to_node.kind, &edge.to_port) else {
                continue;
            };
            let selected = self.state.selected_edge == Some(idx);
            let color = if selected { [255, 210, 120, 255] } else { [132, 160, 210, 255] };
            let mid_x = (from_pos[0] + to_pos[0]) * 0.5;
            push_hline(ui_runtime, from_pos[0], mid_x, from_pos[1], 2.0, color, panel.clip_rect);
            push_vline(ui_runtime, mid_x, from_pos[1], to_pos[1], 2.0, color, panel.clip_rect);
            push_hline(ui_runtime, mid_x, to_pos[0], to_pos[1], 2.0, color, panel.clip_rect);
            if just_pressed
                && (point_near_hline(mouse, from_pos[0], mid_x, from_pos[1], 6.0)
                    || point_near_vline(mouse, mid_x, from_pos[1], to_pos[1], 6.0)
                    || point_near_hline(mouse, mid_x, to_pos[0], to_pos[1], 6.0))
            {
                let dist_sq = edge_polyline_distance_sq(mouse, from_pos, to_pos, mid_x);
                if best_pick.map(|(_, d)| dist_sq < d).unwrap_or(true) {
                    best_pick = Some((idx, dist_sq));
                }
            }
        }
        best_pick.map(|(i, _)| i)
    }

    fn draw_pending_link(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel, origin: [f32; 2], mouse: [f32; 2]) {
        let Some(pending) = self.state.pending_link.as_ref() else {
            return;
        };
        let Some(from_node) = self.state.get_node(pending.from_node) else {
            return;
        };
        let from_size = node_box_size(&from_node.kind);
        let Some(from_pos) = output_port_anchor(origin, from_size, from_node.position, &from_node.kind, &pending.from_port) else {
            return;
        };
        let mid_x = (from_pos[0] + mouse[0]) * 0.5;
        push_hline(ui_runtime, from_pos[0], mid_x, from_pos[1], 2.0, [255, 190, 120, 255], panel.clip_rect);
        push_vline(ui_runtime, mid_x, from_pos[1], mouse[1], 2.0, [255, 190, 120, 255], panel.clip_rect);
        push_hline(ui_runtime, mid_x, mouse[0], mouse[1], 2.0, [255, 190, 120, 255], panel.clip_rect);
    }

    fn draw_footer(&self, ui_runtime: &mut UiRuntime, panel: &UiPanel, toolbar_y: f32) {
        if let Some(selected) = self.state.selected_node
            && let Some(node) = self.state.get_node(selected)
        {
            let text = format!("Selected: {} [{}] Pos({:.0},{:.0})", node.name, node_kind_name(&node.kind), node.position[0], node.position[1]);
            ui_runtime.push_text_ascii(&text, [panel.body_pos[0] + 8.0, panel.body_pos[1] + panel.body_size[1] - 20.0], [8.0, 14.0], [220, 225, 235, 255], panel.clip_rect);
        }
        if let Some(idx) = self.state.selected_edge
            && let Some(edge) = self.state.graph.edges.get(idx)
        {
            let text = format!(
                "Selected Edge: {}.{} -> {}.{}  FB:{}",
                edge.from_node,
                edge.from_port,
                edge.to_node,
                edge.to_port,
                edge.framebuffer
            );
            ui_runtime.push_text_ascii(&text, [panel.body_pos[0] + 8.0, panel.body_pos[1] + panel.body_size[1] - 38.0], [8.0, 14.0], [220, 190, 140, 255], panel.clip_rect);
        }
        if let Some(pending) = self.state.pending_link.as_ref() {
            let text = format!("Linking: {}.{} -> (click input port)", pending.from_node, pending.from_port);
            ui_runtime.push_text_ascii(&text, [panel.body_pos[0] + 960.0, toolbar_y + 8.0], [8.0, 14.0], [255, 220, 160, 255], panel.clip_rect);
        }
        ui_runtime.push_text_ascii(&self.status_text, [panel.body_pos[0] + 8.0, panel.body_pos[1] + panel.body_size[1] - 56.0], [8.0, 14.0], [180, 210, 255, 255], panel.clip_rect);
    }
}

fn edge_polyline_distance_sq(p: [f32; 2], from: [f32; 2], to: [f32; 2], mid_x: f32) -> f32 {
    let d1 = point_to_segment_dist_sq(p, from, [mid_x, from[1]]);
    let d2 = point_to_segment_dist_sq(p, [mid_x, from[1]], [mid_x, to[1]]);
    let d3 = point_to_segment_dist_sq(p, [mid_x, to[1]], to);
    d1.min(d2).min(d3)
}

fn point_to_segment_dist_sq(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let abx = b[0] - a[0];
    let aby = b[1] - a[1];
    let apx = p[0] - a[0];
    let apy = p[1] - a[1];
    let ab_len_sq = abx * abx + aby * aby;
    if ab_len_sq < 1e-8 {
        let dx = p[0] - a[0];
        let dy = p[1] - a[1];
        return dx * dx + dy * dy;
    }
    let t = ((apx * abx + apy * aby) / ab_len_sq).clamp(0.0, 1.0);
    let cx = a[0] + t * abx;
    let cy = a[1] + t * aby;
    let dx = p[0] - cx;
    let dy = p[1] - cy;
    dx * dx + dy * dy
}

fn point_near_hline(p: [f32; 2], x0: f32, x1: f32, y: f32, tolerance: f32) -> bool {
    let min_x = x0.min(x1) - tolerance;
    let max_x = x0.max(x1) + tolerance;
    p[0] >= min_x && p[0] <= max_x && (p[1] - y).abs() <= tolerance
}

fn point_near_vline(p: [f32; 2], x: f32, y0: f32, y1: f32, tolerance: f32) -> bool {
    let min_y = y0.min(y1) - tolerance;
    let max_y = y0.max(y1) + tolerance;
    p[1] >= min_y && p[1] <= max_y && (p[0] - x).abs() <= tolerance
}

fn trim_label(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let keep = max_chars - 3;
    let mut out = String::with_capacity(max_chars);
    for (i, c) in input.chars().enumerate() {
        if i >= keep {
            break;
        }
        out.push(c);
    }
    out.push_str("...");
    out
}

fn node_kind_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::BasePass => "BasePass",
        NodeKind::DeferredLighting => "Deferred",
        NodeKind::ForwardLighting => "Forward",
        NodeKind::DirectionalShadow => "DirShadow",
        NodeKind::PointShadow => "PointShadow",
        NodeKind::MainCamera => "MainCamera",
        NodeKind::ToneMapping => "ToneMap",
        NodeKind::ColorGrading => "ColorGrade",
        NodeKind::Fxaa => "FXAA",
        NodeKind::UiPass => "UI",
        NodeKind::CombineUi => "Combine",
    }
}

fn input_port_anchor(
    origin: [f32; 2],
    node_pos: [f32; 2],
    kind: &NodeKind,
    port_name: &str,
) -> Option<[f32; 2]> {
    let ports = input_ports(kind);
    let idx = ports.iter().position(|p| p.name == port_name)?;
    Some([
        origin[0] + node_pos[0],
        origin[1] + node_pos[1] + 60.0 + idx as f32 * 18.0,
    ])
}

fn output_port_anchor(
    origin: [f32; 2],
    node_size: [f32; 2],
    node_pos: [f32; 2],
    kind: &NodeKind,
    port_name: &str,
) -> Option<[f32; 2]> {
    let ports = output_ports(kind);
    let idx = ports.iter().position(|p| p.name == port_name)?;
    Some([
        origin[0] + node_pos[0] + node_size[0],
        origin[1] + node_pos[1] + 60.0 + idx as f32 * 18.0,
    ])
}

fn node_box_size(kind: &NodeKind) -> [f32; 2] {
    let width = 260.0;
    let port_count = input_ports(kind).len().max(output_ports(kind).len()) as f32;
    let dynamic_height = 60.0 + port_count * 18.0 + 12.0;
    [width, dynamic_height.max(96.0)]
}

fn push_hline(
    ui_runtime: &mut UiRuntime,
    x0: f32,
    x1: f32,
    y: f32,
    thickness: f32,
    color: [u8; 4],
    clip_rect: [f32; 4],
) {
    let min_x = x0.min(x1);
    let width = (x1 - x0).abs().max(1.0);
    ui_runtime.push_colored_rect(
        [min_x, y - thickness * 0.5],
        [width, thickness],
        color,
        clip_rect,
    );
}

fn push_vline(
    ui_runtime: &mut UiRuntime,
    x: f32,
    y0: f32,
    y1: f32,
    thickness: f32,
    color: [u8; 4],
    clip_rect: [f32; 4],
) {
    let min_y = y0.min(y1);
    let height = (y1 - y0).abs().max(1.0);
    ui_runtime.push_colored_rect(
        [x - thickness * 0.5, min_y],
        [thickness, height],
        color,
        clip_rect,
    );
}

fn draw_node_highlight(
    ui_runtime: &mut UiRuntime,
    pos: [f32; 2],
    size: [f32; 2],
    clip_rect: [f32; 4],
) {
    let color = [255, 215, 120, 255];
    let t = 2.0;
    ui_runtime.push_colored_rect([pos[0], pos[1]], [size[0], t], color, clip_rect);
    ui_runtime.push_colored_rect([pos[0], pos[1] + size[1] - t], [size[0], t], color, clip_rect);
    ui_runtime.push_colored_rect([pos[0], pos[1]], [t, size[1]], color, clip_rect);
    ui_runtime.push_colored_rect([pos[0] + size[0] - t, pos[1]], [t, size[1]], color, clip_rect);
}
