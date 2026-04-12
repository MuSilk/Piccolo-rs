use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// 顶点输入拓扑（与 `VkPipelineInputAssemblyStateCreateInfo`、draw 调用一致）。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ShaderVertexInputKind {
    /// 无顶点缓冲，全屏：`TRIANGLE_STRIP` + `cmd_draw(3,1,0,0)`（与现有后处理 Pass 一致）。
    #[default]
    FullscreenTriangleStrip,
}

/// 子通道输入：端口名（与图边 `to_port` 一致）+ 与 SPIR-V / 描述符布局对齐的 binding 与 `input_attachment_index`。
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ShaderSubpassInputSpec {
    pub name: String,
    /// `layout(set = 0, binding = …)`；省略时等于 `input_attachment_index`（若亦省略则等于在列表中的下标）。
    #[serde(default)]
    pub binding: Option<u32>,
    /// `layout(input_attachment_index = …)`，须与本 subpass 的 `pInputAttachments` 顺序一致（从 0 连续）。
    #[serde(default)]
    pub input_attachment_index: Option<u32>,
}

/// 解析后的子通道输入（已按 `input_attachment_index` 升序排列，且下标与 index 一致）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderSubpassInputResolved {
    pub name: String,
    pub binding: u32,
    pub input_attachment_index: u32,
}

/// 可序列化着色器节点：由编辑器写入 JSON，供图编译器推导 subpass，供运行时构建 `VkPipeline`。
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ShaderNodeSpec {
    /// 相对资产根目录的 vertex SPIR-V 路径，例如 `generated/spv/post_process.vert.spv`。
    #[serde(default)]
    pub vert_spirv: String,
    /// 相对资产根目录的 fragment SPIR-V 路径。
    #[serde(default)]
    pub frag_spirv: String,
    /// 可选：源 GLSL 路径（供离线编译 / 工具链，运行时管线仍读 `*_spirv`）。
    #[serde(default)]
    pub glsl_vert: Option<String>,
    #[serde(default)]
    pub glsl_frag: Option<String>,
    #[serde(default = "default_shader_entry")]
    pub vert_entry: String,
    #[serde(default = "default_shader_entry")]
    pub frag_entry: String,
    /// 兼容旧资产：仅端口名，binding / `input_attachment_index` 均按 0..n-1 递增。
    #[serde(default)]
    pub inputs: Vec<String>,
    /// 非空时优先于 `inputs`：显式声明子通道输入与描述符生成信息。
    #[serde(default)]
    pub subpass_inputs: Vec<ShaderSubpassInputSpec>,
    /// 本 subpass 的 color attachment 出端口名（顺序 = `location` / color attachment 序号）。
    #[serde(default)]
    pub color_outputs: Vec<String>,
    #[serde(default)]
    pub vertex_input: ShaderVertexInputKind,
}

/// 将 `ShaderNodeSpec` 中的 `subpass_inputs` 或 legacy `inputs` 解析为按 `input_attachment_index` 排序的列表（用于拼 `VkSubpassDescription::pInputAttachments` 与 descriptor 写入）。
pub fn resolve_shader_subpass_inputs(spec: &ShaderNodeSpec) -> Result<Vec<ShaderSubpassInputResolved>> {
    let mut v: Vec<ShaderSubpassInputResolved> = Vec::new();
    if !spec.subpass_inputs.is_empty() {
        for (i, s) in spec.subpass_inputs.iter().enumerate() {
            let ia = s.input_attachment_index.unwrap_or(i as u32);
            let b = s.binding.unwrap_or(ia);
            v.push(ShaderSubpassInputResolved {
                name: s.name.clone(),
                binding: b,
                input_attachment_index: ia,
            });
        }
    } else {
        for (i, name) in spec.inputs.iter().enumerate() {
            let i = i as u32;
            v.push(ShaderSubpassInputResolved {
                name: name.clone(),
                binding: i,
                input_attachment_index: i,
            });
        }
    }
    v.sort_by_key(|r| r.input_attachment_index);
    for (expect, r) in v.iter().enumerate() {
        if r.input_attachment_index != expect as u32 {
            bail!(
                "shader subpass inputs: input_attachment_index must be contiguous from 0, expected {expect} got {}",
                r.input_attachment_index
            );
        }
    }
    Ok(v)
}

fn default_shader_entry() -> String {
    "main".to_string()
}

/// 与编辑器、资产 JSON（如 `render_pipeline/main_camera_pass.json`）对齐的节点类型。
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
    /// 通用全屏子通道：端口与着色器由 `shader` 字段描述，由 `graph_compiler` 推导 subpass 附件引用。
    ShaderFullscreen,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderGraphNode {
    pub id: u64,
    pub kind: RenderGraphNodeKind,
    pub name: String,
    #[serde(default)]
    pub position: [f32; 2],
    /// `ShaderFullscreen` 必填；其它 kind 可省略。
    #[serde(default)]
    pub shader: Option<ShaderNodeSpec>,
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
