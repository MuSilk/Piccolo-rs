use serde::{Deserialize, Serialize};

/// 与编辑器 `NodeKind`、资产 JSON（如 `render_pipeline/main_camera_pass.json`）对齐的节点类型。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RenderGraphNodeKind {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderGraphNode {
    pub id: u64,
    pub kind: RenderGraphNodeKind,
    pub name: String,
    #[serde(default)]
    pub position: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderGraphEdge {
    pub from_node: u64,
    #[serde(default)]
    pub from_port: String,
    pub to_node: u64,
    #[serde(default)]
    pub to_port: String,
    #[serde(default)]
    pub framebuffer: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderGraphAsset {
    #[serde(default)]
    pub version: u32,
    pub nodes: Vec<RenderGraphNode>,
    pub edges: Vec<RenderGraphEdge>,
}

/// 拓扑排序后的可执行顺序（仅节点 id 与类型；边与 `framebuffer` 仍保留在 `RenderGraphAsset` 中供后续绑定）。
#[derive(Clone, Debug, Default)]
pub struct CompiledRenderGraph {
    pub execution_order: Vec<u64>,
    pub execution_kinds: Vec<RenderGraphNodeKind>,
}
