use serde::{Deserialize, Serialize};

pub use runtime::function::render::render_graph::{
    RenderGraphAsset, RenderGraphEdge as GraphEdge, RenderGraphNode as GraphNode,
    RenderGraphNodeKind as NodeKind, ShaderNodeSpec, ShaderSubpassInputSpec,
    ShaderVertexInputKind, resolve_shader_subpass_inputs,
};

pub type NodeId = u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    ShadowMap,
    SceneColor,
    UiColor,
    FinalColor,
}

#[derive(Clone, Debug)]
pub struct PortDefOwned {
    pub name: String,
    pub ty: PortType,
}

/// 新建「全屏着色器」节点时的默认 SPIR-V 路径（相对 `asset/`）；运行前请将 `runtime/src/shader/generated/spv/` 下对应文件复制到该目录。
pub fn default_shader_fullscreen_spec() -> ShaderNodeSpec {
    ShaderNodeSpec {
        vert_spirv: "generated/spv/post_process.vert.spv".into(),
        frag_spirv: "generated/spv/tone_mapping.frag.spv".into(),
        subpass_inputs: vec![ShaderSubpassInputSpec {
            name: "hdr".into(),
            binding: Some(0),
            input_attachment_index: Some(0),
        }],
        color_outputs: vec!["ldr".into()],
        ..Default::default()
    }
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
        "ldr" => Some("BACKUP_BUFFER_EVEN"),
        "hdr" => Some("BACKUP_BUFFER_ODD"),
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

pub fn input_ports_for(node: &GraphNode) -> Vec<PortDefOwned> {
    use NodeKind::*;
    match &node.kind {
        ShaderFullscreen => node
            .shader
            .as_ref()
            .map(|s| {
                resolve_shader_subpass_inputs(s)
                    .map(|v| {
                        v.into_iter()
                            .map(|r| PortDefOwned {
                                name: r.name,
                                ty: PortType::SceneColor,
                            })
                            .collect()
                    })
                    .unwrap_or_else(|_| {
                        s.inputs
                            .iter()
                            .map(|n| PortDefOwned {
                                name: n.clone(),
                                ty: PortType::SceneColor,
                            })
                            .collect()
                    })
            })
            .unwrap_or_else(|| {
                vec![PortDefOwned {
                    name: "hdr".into(),
                    ty: PortType::SceneColor,
                }]
            }),
        BasePass => vec![],
        DeferredLighting => vec![
            PortDefOwned {
                name: "normal".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "material".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "base_color".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "depth".into(),
                ty: PortType::SceneColor,
            },
        ],
        ForwardLighting => vec![
            PortDefOwned {
                name: "deferred_lit".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "depth".into(),
                ty: PortType::SceneColor,
            },
        ],
        DirectionalShadow | PointShadow => vec![],
        MainCamera => vec![
            PortDefOwned {
                name: "dir_shadow".into(),
                ty: PortType::ShadowMap,
            },
            PortDefOwned {
                name: "point_shadow".into(),
                ty: PortType::ShadowMap,
            },
        ],
        ToneMapping => vec![PortDefOwned {
            name: "lit_hdr".into(),
            ty: PortType::SceneColor,
        }],
        ColorGrading => vec![PortDefOwned {
            name: "tone_mapped".into(),
            ty: PortType::SceneColor,
        }],
        Fxaa => vec![PortDefOwned {
            name: "graded".into(),
            ty: PortType::SceneColor,
        }],
        UiPass => vec![],
        CombineUi => vec![
            PortDefOwned {
                name: "antialiased".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "ui_color".into(),
                ty: PortType::UiColor,
            },
        ],
    }
}

pub fn output_ports_for(node: &GraphNode) -> Vec<PortDefOwned> {
    use NodeKind::*;
    match &node.kind {
        ShaderFullscreen => node
            .shader
            .as_ref()
            .map(|s| {
                s.color_outputs
                    .iter()
                    .map(|n| PortDefOwned {
                        name: n.clone(),
                        ty: PortType::SceneColor,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![PortDefOwned {
                    name: "ldr".into(),
                    ty: PortType::SceneColor,
                }]
            }),
        BasePass => vec![
            PortDefOwned {
                name: "normal".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "material".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "base_color".into(),
                ty: PortType::SceneColor,
            },
            PortDefOwned {
                name: "depth".into(),
                ty: PortType::SceneColor,
            },
        ],
        DeferredLighting => vec![PortDefOwned {
            name: "deferred_lit".into(),
            ty: PortType::SceneColor,
        }],
        ForwardLighting => vec![PortDefOwned {
            name: "lit_hdr".into(),
            ty: PortType::SceneColor,
        }],
        DirectionalShadow | PointShadow => vec![PortDefOwned {
            name: "shadow_out".into(),
            ty: PortType::ShadowMap,
        }],
        MainCamera => vec![PortDefOwned {
            name: "lit_hdr".into(),
            ty: PortType::SceneColor,
        }],
        ToneMapping => vec![PortDefOwned {
            name: "tone_mapped".into(),
            ty: PortType::SceneColor,
        }],
        ColorGrading => vec![PortDefOwned {
            name: "graded".into(),
            ty: PortType::SceneColor,
        }],
        Fxaa => vec![PortDefOwned {
            name: "antialiased".into(),
            ty: PortType::SceneColor,
        }],
        UiPass => vec![PortDefOwned {
            name: "ui_color".into(),
            ty: PortType::UiColor,
        }],
        CombineUi => vec![PortDefOwned {
            name: "present".into(),
            ty: PortType::FinalColor,
        }],
    }
}
