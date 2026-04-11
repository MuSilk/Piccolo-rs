use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeKind {
    BasePass,
    DeferredLighting,
    ForwardLighting,
    DirectionalShadow,
    PointShadow,
    MainCamera,
    ToneMapping,
    ColorGrading,
    #[serde(alias = "FXAA")]
    Fxaa,
    #[serde(alias = "UI")]
    UiPass,
    #[serde(alias = "CombineUI")]
    CombineUi,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    ShadowMap,
    SceneColor,
    UiColor,
    FinalColor,
}

#[derive(Copy, Clone, Debug)]
pub struct PortDef {
    pub name: &'static str,
    pub ty: PortType,
}

pub type NodeId = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
    pub position: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_node: NodeId,
    #[serde(default)]
    pub from_port: String,
    pub to_node: NodeId,
    #[serde(default)]
    pub to_port: String,
    /// 边所对应的 Framebuffer（附件语义），用于图编译器；与端口同名时通常与 `from_port` 一致。
    #[serde(default)]
    pub framebuffer: String,
}

/// 语义端口名 → 引擎 Framebuffer 附件名（与 `render_pass` 下标对应）。
pub fn semantic_port_framebuffer(port: &str) -> Option<&'static str> {
    match port {
        "normal" => Some("GBUFFER_A"),
        "material" => Some("GBUFFER_B"),
        "base_color" => Some("GBUFFER_C"),
        "depth" => Some("DEPTH"),
        "deferred_lit" => Some("BACKUP_BUFFER_ODD"),
        "lit_hdr" => Some("BACKUP_BUFFER_ODD"),
        "tone_mapped" => Some("BACKUP_BUFFER_EVEN"),
        "graded" => Some("POST_PROCESS_BUFFER_ODD"),
        "antialiased" => Some("BACKUP_BUFFER_ODD"),
        "ui_color" => Some("BACKUP_BUFFER_EVEN"),
        "present" => Some("SWAPCHAIN_IMAGE"),
        _ => None,
    }
}

/// 当 JSON 未写 `framebuffer` 时，根据端口名推断边对应的 Framebuffer。
pub fn infer_edge_framebuffer(from_port: &str, to_port: &str) -> String {
    if is_framebuffer_attachment_name(from_port) {
        return from_port.to_string();
    }
    if is_framebuffer_attachment_name(to_port) {
        return to_port.to_string();
    }
    if let Some(fb) = semantic_port_framebuffer(from_port) {
        return fb.to_string();
    }
    if let Some(fb) = semantic_port_framebuffer(to_port) {
        return fb.to_string();
    }
    match (from_port, to_port) {
        ("shadow_out", "dir_shadow") => "DIRECTIONAL_SHADOW_COLOR".to_string(),
        ("shadow_out", "point_shadow") => "POINT_SHADOW_COLOR".to_string(),
        ("shadow_out", _) => "SHADOW_MAP".to_string(),
        ("scene_out", "scene_in") => "BACKUP_BUFFER_ODD".to_string(),
        ("ui_out", "ui_in") => "BACKUP_BUFFER_EVEN".to_string(),
        _ => from_port.to_string(),
    }
}

fn is_framebuffer_attachment_name(s: &str) -> bool {
    matches!(
        s,
        "GBUFFER_A"
            | "GBUFFER_B"
            | "GBUFFER_C"
            | "DEPTH"
            | "BACKUP_BUFFER_ODD"
            | "BACKUP_BUFFER_EVEN"
            | "POST_PROCESS_BUFFER_ODD"
            | "POST_PROCESS_BUFFER_EVEN"
            | "SWAPCHAIN_IMAGE"
    )
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderGraphAsset {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub fn input_ports(kind: &NodeKind) -> &'static [PortDef] {
    match kind {
        NodeKind::BasePass => &[],
        NodeKind::DeferredLighting => &[
            PortDef {
                name: "normal",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "material",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "base_color",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "depth",
                ty: PortType::SceneColor,
            },
        ],
        NodeKind::ForwardLighting => &[
            PortDef {
                name: "deferred_lit",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "depth",
                ty: PortType::SceneColor,
            },
        ],
        NodeKind::DirectionalShadow => &[],
        NodeKind::PointShadow => &[],
        NodeKind::MainCamera => &[
            PortDef {
                name: "dir_shadow",
                ty: PortType::ShadowMap,
            },
            PortDef {
                name: "point_shadow",
                ty: PortType::ShadowMap,
            },
        ],
        NodeKind::ToneMapping => &[PortDef {
            name: "lit_hdr",
            ty: PortType::SceneColor,
        }],
        NodeKind::ColorGrading => &[PortDef {
            name: "tone_mapped",
            ty: PortType::SceneColor,
        }],
        NodeKind::Fxaa => &[PortDef {
            name: "graded",
            ty: PortType::SceneColor,
        }],
        NodeKind::UiPass => &[],
        NodeKind::CombineUi => &[
            PortDef {
                name: "antialiased",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "ui_color",
                ty: PortType::UiColor,
            },
        ],
    }
}

pub fn output_ports(kind: &NodeKind) -> &'static [PortDef] {
    match kind {
        NodeKind::BasePass => &[
            PortDef {
                name: "normal",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "material",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "base_color",
                ty: PortType::SceneColor,
            },
            PortDef {
                name: "depth",
                ty: PortType::SceneColor,
            },
        ],
        NodeKind::DeferredLighting => &[PortDef {
            name: "deferred_lit",
            ty: PortType::SceneColor,
        }],
        NodeKind::ForwardLighting => &[PortDef {
            name: "lit_hdr",
            ty: PortType::SceneColor,
        }],
        NodeKind::DirectionalShadow => &[PortDef {
            name: "shadow_out",
            ty: PortType::ShadowMap,
        }],
        NodeKind::PointShadow => &[PortDef {
            name: "shadow_out",
            ty: PortType::ShadowMap,
        }],
        NodeKind::MainCamera => &[PortDef {
            name: "lit_hdr",
            ty: PortType::SceneColor,
        }],
        NodeKind::ToneMapping => &[PortDef {
            name: "tone_mapped",
            ty: PortType::SceneColor,
        }],
        NodeKind::ColorGrading => &[PortDef {
            name: "graded",
            ty: PortType::SceneColor,
        }],
        NodeKind::Fxaa => &[PortDef {
            name: "antialiased",
            ty: PortType::SceneColor,
        }],
        NodeKind::UiPass => &[PortDef {
            name: "ui_color",
            ty: PortType::UiColor,
        }],
        NodeKind::CombineUi => &[PortDef {
            name: "present",
            ty: PortType::FinalColor,
        }],
    }
}
