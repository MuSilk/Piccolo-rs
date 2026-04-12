use super::graph_types::{
    GraphEdge, GraphNode, NodeKind, PortType, RenderGraphAsset, default_shader_fullscreen_spec,
    infer_edge_framebuffer, input_ports_for, output_ports_for,
};

#[derive(Clone)]
pub struct PendingLink {
    pub from_node: u64,
    pub from_port: String,
    pub port_type: PortType,
}

#[derive(Default)]
pub struct RenderGraphState {
    pub graph: RenderGraphAsset,
    pub selected_node: Option<u64>,
    pub selected_edge: Option<usize>,
    pub dragging_node: Option<u64>,
    pub drag_offset: [f32; 2],
    pub pending_link: Option<PendingLink>,
    pub next_node_id: u64,
}

impl RenderGraphState {
    pub fn with_default_graph() -> Self {
        let nodes = vec![
            GraphNode {
                id: 1,
                kind: NodeKind::DirectionalShadow,
                name: "Directional Shadow".to_string(),
                position: [24.0, 28.0],
                shader: None,
            },
            GraphNode {
                id: 2,
                kind: NodeKind::MainCamera,
                name: "Main Camera".to_string(),
                position: [340.0, 28.0],
                shader: None,
            },
            GraphNode {
                id: 3,
                kind: NodeKind::ToneMapping,
                name: "Tone Mapping".to_string(),
                position: [656.0, 28.0],
                shader: None,
            },
            GraphNode {
                id: 4,
                kind: NodeKind::ColorGrading,
                name: "Color Grading".to_string(),
                position: [972.0, 28.0],
                shader: None,
            },
            GraphNode {
                id: 5,
                kind: NodeKind::Fxaa,
                name: "FXAA".to_string(),
                position: [1288.0, 28.0],
                shader: None,
            },
            GraphNode {
                id: 6,
                kind: NodeKind::UiPass,
                name: "UI Pass".to_string(),
                position: [1288.0, 220.0],
                shader: None,
            },
            GraphNode {
                id: 7,
                kind: NodeKind::CombineUi,
                name: "Combine UI".to_string(),
                position: [1604.0, 126.0],
                shader: None,
            },
            GraphNode {
                id: 8,
                kind: NodeKind::PointShadow,
                name: "Point Shadow".to_string(),
                position: [24.0, 220.0],
                shader: None,
            },
        ];
        let edges = vec![
            GraphEdge {
                from_node: 1,
                from_port: "shadow_out".to_string(),
                to_node: 2,
                to_port: "dir_shadow".to_string(),
                framebuffer: "DIRECTIONAL_SHADOW_COLOR".to_string(),
            },
            GraphEdge {
                from_node: 8,
                from_port: "shadow_out".to_string(),
                to_node: 2,
                to_port: "point_shadow".to_string(),
                framebuffer: "POINT_SHADOW_COLOR".to_string(),
            },
            GraphEdge {
                from_node: 2,
                from_port: "lit_hdr".to_string(),
                to_node: 3,
                to_port: "lit_hdr".to_string(),
                framebuffer: "BACKUP_BUFFER_ODD".to_string(),
            },
            GraphEdge {
                from_node: 3,
                from_port: "tone_mapped".to_string(),
                to_node: 4,
                to_port: "tone_mapped".to_string(),
                framebuffer: "BACKUP_BUFFER_EVEN".to_string(),
            },
            GraphEdge {
                from_node: 4,
                from_port: "graded".to_string(),
                to_node: 5,
                to_port: "graded".to_string(),
                framebuffer: "POST_PROCESS_BUFFER_ODD".to_string(),
            },
            GraphEdge {
                from_node: 5,
                from_port: "antialiased".to_string(),
                to_node: 7,
                to_port: "antialiased".to_string(),
                framebuffer: "BACKUP_BUFFER_ODD".to_string(),
            },
            GraphEdge {
                from_node: 6,
                from_port: "ui_color".to_string(),
                to_node: 7,
                to_port: "ui_color".to_string(),
                framebuffer: "BACKUP_BUFFER_EVEN".to_string(),
            },
        ];
        Self {
            graph: RenderGraphAsset {
                version: 1,
                nodes,
                edges,
            },
            selected_node: None,
            selected_edge: None,
            dragging_node: None,
            drag_offset: [0.0, 0.0],
            pending_link: None,
            next_node_id: 9,
        }
    }

    pub fn add_node(&mut self) {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.graph.nodes.push(GraphNode {
            id,
            kind: NodeKind::ShaderFullscreen,
            name: format!("Shader {id}"),
            position: [120.0, 120.0],
            shader: Some(default_shader_fullscreen_spec()),
        });
        self.selected_node = Some(id);
        self.selected_edge = None;
    }

    pub fn delete_selected_node(&mut self) {
        let Some(id) = self.selected_node else {
            return;
        };
        self.graph.nodes.retain(|n| n.id != id);
        self.graph
            .edges
            .retain(|e| e.from_node != id && e.to_node != id);
        self.selected_node = None;
        self.selected_edge = None;
        if self.dragging_node == Some(id) {
            self.dragging_node = None;
        }
        if self
            .pending_link
            .as_ref()
            .map(|x| x.from_node == id)
            .unwrap_or(false)
        {
            self.pending_link = None;
        }
    }

    pub fn get_node_mut(&mut self, id: u64) -> Option<&mut GraphNode> {
        self.graph.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn get_node(&self, id: u64) -> Option<&GraphNode> {
        self.graph.nodes.iter().find(|n| n.id == id)
    }

    pub fn begin_link_from_output(&mut self, from_node: u64, from_port: &str) -> Result<(), String> {
        let Some(node) = self.get_node(from_node) else {
            return Err("source node not found".to_string());
        };
        let outputs = output_ports_for(node);
        let Some(port) = outputs.iter().find(|p| p.name == from_port) else {
            return Err("output port not found".to_string());
        };
        self.pending_link = Some(PendingLink {
            from_node,
            from_port: from_port.to_string(),
            port_type: port.ty,
        });
        Ok(())
    }

    pub fn cancel_link(&mut self) {
        self.pending_link = None;
    }

    pub fn link_pending_to_input(&mut self, to_node: u64, to_port: &str) -> Result<(), String> {
        let Some(pending) = self.pending_link.clone() else {
            return Err("no pending output link".to_string());
        };
        if pending.from_node == to_node {
            self.pending_link = None;
            return Err("cannot link node to itself".to_string());
        }
        let Some(target_node) = self.get_node(to_node) else {
            self.pending_link = None;
            return Err("target node not found".to_string());
        };
        let inputs = input_ports_for(target_node);
        let Some(input) = inputs.iter().find(|p| p.name == to_port) else {
            self.pending_link = None;
            return Err("input port not found".to_string());
        };
        if input.ty != pending.port_type {
            self.pending_link = None;
            return Err("port type mismatch".to_string());
        }
        if self.path_exists(to_node, pending.from_node) {
            self.pending_link = None;
            return Err("link would create a cycle".to_string());
        }
        let duplicate = self.graph.edges.iter().any(|e| {
            e.from_node == pending.from_node
                && e.from_port == pending.from_port
                && e.to_node == to_node
                && e.to_port == to_port
        });
        if duplicate {
            self.pending_link = None;
            return Err("edge already exists".to_string());
        }
        let input_used = self
            .graph
            .edges
            .iter()
            .any(|e| e.to_node == to_node && e.to_port == to_port);
        if input_used {
            self.pending_link = None;
            return Err("input port already connected".to_string());
        }
        let framebuffer =
            infer_edge_framebuffer(&pending.from_port, to_port);
        self.graph.edges.push(GraphEdge {
            from_node: pending.from_node,
            from_port: pending.from_port,
            to_node,
            to_port: to_port.to_string(),
            framebuffer,
        });
        self.pending_link = None;
        Ok(())
    }

    pub fn select_node(&mut self, id: u64) {
        self.selected_node = Some(id);
        self.selected_edge = None;
    }

    /// 按边在 `edges` 中的下标选中（唯一区分同节点对之间的多条边）。
    pub fn select_edge_index(&mut self, idx: usize) {
        if idx < self.graph.edges.len() {
            self.selected_edge = Some(idx);
            self.selected_node = None;
        }
    }

    pub fn delete_selected_edge(&mut self) {
        let Some(idx) = self.selected_edge else {
            return;
        };
        if idx < self.graph.edges.len() {
            self.graph.edges.remove(idx);
        }
        self.selected_edge = None;
    }

    pub fn cycle_selected_node_kind(&mut self) {
        let Some(id) = self.selected_node else {
            return;
        };
        let Some(node) = self.get_node_mut(id) else {
            return;
        };
        let prev = node.kind.clone();
        node.kind = next_kind(&prev);
        if matches!(node.kind, NodeKind::ShaderFullscreen) && node.shader.is_none() {
            node.shader = Some(default_shader_fullscreen_spec());
        }
        if matches!(prev, NodeKind::ShaderFullscreen) && !matches!(node.kind, NodeKind::ShaderFullscreen) {
            node.shader = None;
        }
    }

    pub fn replace_graph(&mut self, graph: RenderGraphAsset) {
        self.graph = graph;
        self.selected_node = None;
        self.selected_edge = None;
        self.dragging_node = None;
        self.pending_link = None;
        self.drag_offset = [0.0, 0.0];
        self.normalize_edges_ports();
        self.next_node_id = self
            .graph
            .nodes
            .iter()
            .map(|n| n.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
    }

    fn normalize_edges_ports(&mut self) {
        for edge in &mut self.graph.edges {
            if edge.from_port.is_empty()
                && let Some(from_node) = self.graph.nodes.iter().find(|n| n.id == edge.from_node)
                && let Some(port) = output_ports_for(from_node).first()
            {
                edge.from_port = port.name.to_string();
            }
            if edge.to_port.is_empty()
                && let Some(to_node) = self.graph.nodes.iter().find(|n| n.id == edge.to_node)
                && let Some(port) = input_ports_for(to_node).first()
            {
                edge.to_port = port.name.to_string();
            }
            if edge.framebuffer.is_empty() {
                edge.framebuffer = infer_edge_framebuffer(&edge.from_port, &edge.to_port);
            }
        }
    }
}

fn next_kind(kind: &NodeKind) -> NodeKind {
    match kind {
        NodeKind::BasePass => NodeKind::DeferredLighting,
        NodeKind::DeferredLighting => NodeKind::ForwardLighting,
        NodeKind::ForwardLighting => NodeKind::DirectionalShadow,
        NodeKind::DirectionalShadow => NodeKind::PointShadow,
        NodeKind::PointShadow => NodeKind::MainCamera,
        NodeKind::MainCamera => NodeKind::ToneMapping,
        NodeKind::ToneMapping => NodeKind::ColorGrading,
        NodeKind::ColorGrading => NodeKind::Fxaa,
        NodeKind::Fxaa => NodeKind::UiPass,
        NodeKind::UiPass => NodeKind::CombineUi,
        NodeKind::CombineUi => NodeKind::ShaderFullscreen,
        NodeKind::ShaderFullscreen => NodeKind::DirectionalShadow,
    }
}

impl RenderGraphState {
    fn path_exists(&self, start: u64, target: u64) -> bool {
        if start == target {
            return true;
        }
        let mut stack = vec![start];
        let mut visited = std::collections::HashSet::new();
        while let Some(cur) = stack.pop() {
            if !visited.insert(cur) {
                continue;
            }
            for edge in &self.graph.edges {
                if edge.from_node == cur {
                    if edge.to_node == target {
                        return true;
                    }
                    stack.push(edge.to_node);
                }
            }
        }
        false
    }
}
