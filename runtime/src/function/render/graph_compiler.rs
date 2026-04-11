use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use anyhow::{Result, anyhow, bail};

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;
use crate::function::render::render_graph::{CompiledRenderGraph, RenderGraphAsset, RenderGraphEdge, RenderGraphNodeKind};
use crate::function::render::render_pass::{
    _MAIN_CAMERA_PASS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD,
    _MAIN_CAMERA_PASS_DEPTH, _MAIN_CAMERA_PASS_GBUFFER_A, _MAIN_CAMERA_PASS_GBUFFER_B, _MAIN_CAMERA_PASS_GBUFFER_C,
    _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD, _MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE,
    FrameBufferAttachment,
};
use crate::resource::config_manager::ConfigManager;
use vulkanalia::{prelude::v1_0::*};

pub const DEFAULT_RENDER_GRAPH_RELATIVE_PATH: &str = "render_graph/default_graph.json";

/// 与 `MainCameraPass` 子通道顺序对应的展开图（无 `MainCamera` 聚合节点）。
pub const MAIN_CAMERA_PASS_GRAPH_RELATIVE_PATH: &str = "render_pipeline/main_camera_pass.json";

pub fn load_render_graph_from_relative(
    config_manager: &ConfigManager,
    relative_to_asset_folder: &str,
) -> Result<RenderGraphAsset> {
    let file_path = config_manager.get_asset_folder().join(relative_to_asset_folder);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| anyhow!("read render graph file failed ({}): {e}", file_path.display()))?;
    let graph = serde_json::from_str::<RenderGraphAsset>(&content)
        .map_err(|e| anyhow!("parse render graph json failed ({}): {e}", file_path.display()))?;
    Ok(graph)
}

pub fn load_render_graph_asset(config_manager: &ConfigManager) -> Result<RenderGraphAsset> {
    load_render_graph_from_relative(config_manager, DEFAULT_RENDER_GRAPH_RELATIVE_PATH)
}

pub fn load_main_camera_pass_graph(config_manager: &ConfigManager) -> Result<RenderGraphAsset> {
    load_render_graph_from_relative(config_manager, MAIN_CAMERA_PASS_GRAPH_RELATIVE_PATH)
}

/// 将资产反序列化为 `RenderGraphAsset` 并做 JSON 语法校验（不读文件）。
pub fn parse_render_graph_json(json: &str) -> Result<RenderGraphAsset> {
    serde_json::from_str(json).map_err(|e| anyhow!("parse render graph json failed: {e}"))
}

/// 拓扑排序；多条边指向同一节点会正确增加入度。使用 **节点 id 升序** 作为同层就绪顺序，保证结果稳定。
pub fn compile_render_graph(graph: &RenderGraphAsset) -> Result<CompiledRenderGraph> {
    if graph.nodes.is_empty() {
        bail!("render graph has no nodes");
    }

    let mut node_kind_by_id: HashMap<u64, RenderGraphNodeKind> = HashMap::new();
    for node in &graph.nodes {
        if node_kind_by_id.insert(node.id, node.kind.clone()).is_some() {
            bail!("duplicate node id {}", node.id);
        }
    }

    let mut indegree: HashMap<u64, usize> = node_kind_by_id.keys().map(|id| (*id, 0)).collect();
    let mut out_edges: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &graph.edges {
        if !node_kind_by_id.contains_key(&edge.from_node) {
            bail!("edge has unknown from_node {}", edge.from_node);
        }
        if !node_kind_by_id.contains_key(&edge.to_node) {
            bail!("edge has unknown to_node {}", edge.to_node);
        }
        out_edges.entry(edge.from_node).or_default().push(edge.to_node);
        if let Some(x) = indegree.get_mut(&edge.to_node) {
            *x += 1;
        }
    }

    let mut ready: BinaryHeap<Reverse<u64>> = indegree
        .iter()
        .filter_map(|(id, deg)| (*deg == 0).then_some(Reverse(*id)))
        .collect();

    let mut order: Vec<u64> = Vec::with_capacity(node_kind_by_id.len());
    while let Some(Reverse(node)) = ready.pop() {
        order.push(node);
        if let Some(targets) = out_edges.get(&node) {
            for to in targets {
                if let Some(deg) = indegree.get_mut(to) {
                    *deg -= 1;
                    if *deg == 0 {
                        ready.push(Reverse(*to));
                    }
                }
            }
        }
    }

    if order.len() != node_kind_by_id.len() {
        bail!("render graph contains cycle");
    }

    let execution_kinds = order
        .iter()
        .filter_map(|id| node_kind_by_id.get(id))
        .cloned()
        .collect::<Vec<_>>();

    Ok(CompiledRenderGraph {
        execution_order: order,
        execution_kinds,
    })
}

/// 加载并编译 `MAIN_CAMERA_PASS_GRAPH_RELATIVE_PATH` 指向的图。
pub fn load_and_compile_main_camera_pass_graph(
    config_manager: &ConfigManager,
) -> Result<CompiledRenderGraph> {
    let asset = load_main_camera_pass_graph(config_manager)?;
    compile_render_graph(&asset)
}

/// 使用 `fb_attachments` 中的 **format**（与 `MainCameraPass::setup_render_pass` 一致），按 `CompiledRenderGraph` 的拓扑顺序
/// 生成子通道与依赖，创建 Vulkan `RenderPass`。
///
/// 各子通道的 color / input attachment 索引由 **`asset.edges[*].framebuffer`** 解析（与端口 `from_port` / `to_port` 对应），
pub fn build_subpass(
    rhi: &VulkanRHI,
    fb_attachments: &[FrameBufferAttachment],
    asset: &RenderGraphAsset,
) -> Result<vk::RenderPass> {
    let compiled = compile_render_graph(asset)?;
    if compiled.execution_kinds.is_empty() {
        bail!("compiled graph has no execution_kinds");
    }
    if compiled.execution_order.len() != compiled.execution_kinds.len() {
        bail!("execution_order / execution_kinds length mismatch");
    }

    let attachment_descs = build_main_camera_attachment_descriptions(rhi, fb_attachments)?;

    let subpass_layouts: Vec<SubpassLayout> = compiled
        .execution_kinds
        .iter()
        .map(|k| subpass_layout(k))
        .collect::<Result<_>>()?;

    let mut color_refs: Vec<Vec<vk::AttachmentReference>> = Vec::new();
    let mut input_refs: Vec<Vec<vk::AttachmentReference>> = Vec::new();
    let mut depth_refs: Vec<Option<vk::AttachmentReference>> = Vec::new();
    let mut preserve_refs: Vec<Vec<u32>> = Vec::new();

    for (subpass_idx, &node_id) in compiled.execution_order.iter().enumerate() {
        push_subpass_attachment_refs_for_node(
            subpass_idx,
            node_id,
            &compiled.execution_order,
            &subpass_layouts,
            &asset.edges,
            &mut color_refs,
            &mut input_refs,
            &mut depth_refs,
            &mut preserve_refs,
        )?;
    }

    let mut subpasses: Vec<vk::SubpassDescription> = Vec::with_capacity(compiled.execution_kinds.len());
    for i in 0..compiled.execution_kinds.len() {
        let mut b = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs[i]);
        if !input_refs[i].is_empty() {
            b = b.input_attachments(&input_refs[i]);
        }
        if let Some(ref d) = depth_refs[i] {
            b = b.depth_stencil_attachment(d);
        }
        if !preserve_refs[i].is_empty() {
            b = b.preserve_attachments(&preserve_refs[i]);
        }
        subpasses.push(b.build());
    }

    let dependencies = build_linear_subpass_dependencies(subpasses.len());
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachment_descs)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    rhi.create_render_pass(&info)
}

fn build_main_camera_attachment_descriptions(
    rhi: &VulkanRHI,
    fb: &[FrameBufferAttachment],
) -> Result<[vk::AttachmentDescription; _MAIN_CAMERA_PASS_ATTACHMENT_COUNT]> {
    let mut attachments = [vk::AttachmentDescription::default(); _MAIN_CAMERA_PASS_ATTACHMENT_COUNT];

    attachments[_MAIN_CAMERA_PASS_GBUFFER_A] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_GBUFFER_A].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_GBUFFER_B] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_GBUFFER_B].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_GBUFFER_C] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_GBUFFER_C].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN] = vk::AttachmentDescription::builder()
        .format(fb[_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN].format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_DEPTH] = vk::AttachmentDescription::builder()
        .format(rhi.get_depth_image_info().format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    attachments[_MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE] = vk::AttachmentDescription::builder()
        .format(rhi.get_swapchain_info().image_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    Ok(attachments)
}

/// 描述「端口名 ↔ Vulkan 子通道角色」；color 出端口优先由出边 `framebuffer` 解析，无出边时见 `color_out_attachment_index` 内建规则。
#[derive(Clone, Copy)]
struct SubpassLayout {
    color_out_ports: &'static [&'static str],
    input_ports: &'static [&'static str],
    depth: DepthSlot,
}

#[derive(Clone, Copy)]
enum DepthSlot {
    None,
    OutgoingFromPort(&'static str),
    IncomingToPort(&'static str),
}

fn subpass_layout(kind: &RenderGraphNodeKind) -> Result<SubpassLayout> {
    use RenderGraphNodeKind::*;
    Ok(match kind {
        BasePass => SubpassLayout {
            color_out_ports: &["normal", "material", "base_color"],
            input_ports: &[],
            depth: DepthSlot::OutgoingFromPort("depth"),
        },
        DeferredLighting => SubpassLayout {
            color_out_ports: &["deferred_lit"],
            input_ports: &["normal", "material", "base_color", "depth"],
            depth: DepthSlot::None,
        },
        ForwardLighting => SubpassLayout {
            color_out_ports: &["lit_hdr"],
            input_ports: &[],
            depth: DepthSlot::IncomingToPort("depth"),
        },
        ToneMapping => SubpassLayout {
            color_out_ports: &["tone_mapped"],
            input_ports: &["lit_hdr"],
            depth: DepthSlot::None,
        },
        ColorGrading => SubpassLayout {
            color_out_ports: &["graded"],
            input_ports: &["tone_mapped"],
            depth: DepthSlot::None,
        },
        Fxaa => SubpassLayout {
            color_out_ports: &["antialiased"],
            input_ports: &["graded"],
            depth: DepthSlot::None,
        },
        UiPass => SubpassLayout {
            color_out_ports: &["ui_color"],
            input_ports: &[],
            depth: DepthSlot::None,
        },
        CombineUi => SubpassLayout {
            color_out_ports: &["present"],
            input_ports: &["antialiased", "ui_color"],
            depth: DepthSlot::None,
        },
        MainCamera | DirectionalShadow | PointShadow => {
            bail!("build_subpass: unsupported node kind in graph: {kind:?}");
        }
    })
}

/// 本 subpass **之前**（不含当前）已写入过的 attachment 索引（由出边 + 固定 RT + depth 出边推断）。
fn attachment_indices_written_before_subpass(
    execution_order: &[u64],
    edges: &[RenderGraphEdge],
    layouts: &[SubpassLayout],
    subpass_index_exclusive_end: usize,
) -> Result<HashSet<usize>> {
    let mut s = HashSet::new();
    for i in 0..subpass_index_exclusive_end {
        s.extend(attachment_indices_written_by_subpass(
            edges,
            execution_order[i],
            layouts[i],
        )?);
    }
    Ok(s)
}

/// 本 subpass **之后** 子图中，作为某条入边 `framebuffer` 被读到的 attachment 索引。
fn attachment_indices_read_after_subpass(
    execution_order: &[u64],
    edges: &[RenderGraphEdge],
    subpass_index: usize,
) -> Result<HashSet<usize>> {
    let mut s = HashSet::new();
    for &nid in execution_order.iter().skip(subpass_index + 1) {
        for e in edges.iter().filter(|e| e.to_node == nid) {
            s.insert(framebuffer_name_to_index(&e.framebuffer)?);
        }
    }
    Ok(s)
}

fn attachment_indices_written_by_subpass(
    edges: &[RenderGraphEdge],
    node_id: u64,
    layout: SubpassLayout,
) -> Result<HashSet<usize>> {
    let mut out = HashSet::new();
    for &port in layout.color_out_ports {
        out.insert(color_out_attachment_index(edges, node_id, port)?);
    }
    if let DepthSlot::OutgoingFromPort(port) = layout.depth {
        let e = find_edge_from(edges, node_id, port).ok_or_else(|| {
            anyhow!(
                "no outgoing depth edge from node {} port {:?}",
                node_id,
                port
            )
        })?;
        out.insert(framebuffer_name_to_index(&e.framebuffer)?);
    }
    Ok(out)
}

/// 当前 subpass 作为 color / input / depth 实际用到的 attachment（用于从 preserve 候选中剔除）。
fn attachment_indices_used_this_subpass(
    edges: &[RenderGraphEdge],
    node_id: u64,
    layout: SubpassLayout,
) -> Result<HashSet<usize>> {
    let mut u = attachment_indices_written_by_subpass(edges, node_id, layout)?;
    for &port in layout.input_ports {
        let e = find_edge_to(edges, node_id, port).ok_or_else(|| {
            anyhow!("no incoming edge to node {} port {:?}", node_id, port)
        })?;
        u.insert(framebuffer_name_to_index(&e.framebuffer)?);
    }
    if let DepthSlot::IncomingToPort(port) = layout.depth {
        let e = find_edge_to(edges, node_id, port).ok_or_else(|| {
            anyhow!("no incoming depth edge to node {} port {:?}", node_id, port)
        })?;
        u.insert(framebuffer_name_to_index(&e.framebuffer)?);
    }
    Ok(u)
}

/// Vulkan `preserve_attachments`：此前已写入、且后续仍会被读，但 **本 subpass 未直接使用** 的附件。
fn compute_preserve_attachments(
    execution_order: &[u64],
    edges: &[RenderGraphEdge],
    subpass_index: usize,
    used: &HashSet<usize>,
    layouts: &[SubpassLayout],
) -> Result<Vec<u32>> {
    let written_before = attachment_indices_written_before_subpass(
        execution_order,
        edges,
        layouts,
        subpass_index,
    )?;
    let read_after = attachment_indices_read_after_subpass(execution_order, edges, subpass_index)?;
    let mut v: Vec<u32> = written_before
        .intersection(&read_after)
        .filter(|idx| !used.contains(idx))
        .map(|&idx| idx as u32)
        .collect();
    v.sort_unstable();
    Ok(v)
}

fn framebuffer_name_to_index(name: &str) -> Result<usize> {
    let n = name.trim();
    match n {
        "GBUFFER_A" => Ok(_MAIN_CAMERA_PASS_GBUFFER_A),
        "GBUFFER_B" => Ok(_MAIN_CAMERA_PASS_GBUFFER_B),
        "GBUFFER_C" => Ok(_MAIN_CAMERA_PASS_GBUFFER_C),
        "BACKUP_BUFFER_ODD" => Ok(_MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD),
        "BACKUP_BUFFER_EVEN" => Ok(_MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN),
        "POST_PROCESS_BUFFER_ODD" => Ok(_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD),
        "POST_PROCESS_BUFFER_EVEN" => Ok(_MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN),
        "DEPTH" => Ok(_MAIN_CAMERA_PASS_DEPTH),
        "SWAPCHAIN_IMAGE" => Ok(_MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE),
        "" => bail!("edge has empty framebuffer"),
        _ => bail!("unknown framebuffer name on edge: {n:?}"),
    }
}

fn find_edge_from<'a>(edges: &'a [RenderGraphEdge], from_node: u64, from_port: &str) -> Option<&'a RenderGraphEdge> {
    edges.iter().find(|e| e.from_node == from_node && e.from_port == from_port)
}

fn find_edge_to<'a>(edges: &'a [RenderGraphEdge], to_node: u64, to_port: &str) -> Option<&'a RenderGraphEdge> {
    edges.iter().find(|e| e.to_node == to_node && e.to_port == to_port)
}

/// Color RT 出端口：优先 `from_node`/`from_port` 出边的 `framebuffer`；无出边时对少量内建端口解析（如 `present` → swapchain）。
fn color_out_attachment_index(edges: &[RenderGraphEdge], from_node: u64, from_port: &str) -> Result<usize> {
    if let Some(e) = find_edge_from(edges, from_node, from_port) {
        return framebuffer_name_to_index(&e.framebuffer);
    }
    match from_port {
        "present" => Ok(_MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE),
        _ => bail!(
            "no outgoing edge from node {} port {:?}",
            from_node,
            from_port
        ),
    }
}

fn attachment_ref_color_out(edges: &[RenderGraphEdge], from_node: u64, from_port: &str) -> Result<vk::AttachmentReference> {
    let idx = color_out_attachment_index(edges, from_node, from_port)?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    })
}

fn attachment_ref_input_read(edges: &[RenderGraphEdge], to_node: u64, to_port: &str) -> Result<vk::AttachmentReference> {
    let e = find_edge_to(edges, to_node, to_port).ok_or_else(|| {
        anyhow!(
            "no incoming edge to node {} port {:?}",
            to_node,
            to_port
        )
    })?;
    let idx = framebuffer_name_to_index(&e.framebuffer)?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    })
}

fn attachment_ref_depth_stencil_out_port(
    edges: &[RenderGraphEdge],
    from_node: u64,
    from_port: &str,
) -> Result<vk::AttachmentReference> {
    let e = find_edge_from(edges, from_node, from_port).ok_or_else(|| {
        anyhow!(
            "no outgoing depth edge from node {} port {:?}",
            from_node,
            from_port
        )
    })?;
    let idx = framebuffer_name_to_index(&e.framebuffer)?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    })
}

fn attachment_ref_depth_stencil_in_port(
    edges: &[RenderGraphEdge],
    to_node: u64,
    to_port: &str,
) -> Result<vk::AttachmentReference> {
    let e = find_edge_to(edges, to_node, to_port).ok_or_else(|| {
        anyhow!(
            "no incoming depth edge to node {} port {:?}",
            to_node,
            to_port
        )
    })?;
    let idx = framebuffer_name_to_index(&e.framebuffer)?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    })
}

fn push_subpass_attachment_refs_for_node(
    subpass_index: usize,
    node_id: u64,
    execution_order: &[u64],
    layouts: &[SubpassLayout],
    edges: &[RenderGraphEdge],
    color_refs: &mut Vec<Vec<vk::AttachmentReference>>,
    input_refs: &mut Vec<Vec<vk::AttachmentReference>>,
    depth_refs: &mut Vec<Option<vk::AttachmentReference>>,
    preserve_refs: &mut Vec<Vec<u32>>,
) -> Result<()> {
    let layout = layouts[subpass_index];

    let mut colors = Vec::new();
    for &port in layout.color_out_ports {
        colors.push(attachment_ref_color_out(edges, node_id, port)?);
    }
    color_refs.push(colors);

    let mut inputs = Vec::new();
    for &port in layout.input_ports {
        inputs.push(attachment_ref_input_read(edges, node_id, port)?);
    }
    input_refs.push(inputs);

    let depth = match layout.depth {
        DepthSlot::None => None,
        DepthSlot::OutgoingFromPort(port) => Some(attachment_ref_depth_stencil_out_port(
            edges, node_id, port,
        )?),
        DepthSlot::IncomingToPort(port) => Some(attachment_ref_depth_stencil_in_port(
            edges, node_id, port,
        )?),
    };
    depth_refs.push(depth);

    let used = attachment_indices_used_this_subpass(edges, node_id, layout)?;
    preserve_refs.push(compute_preserve_attachments(
        execution_order,
        edges,
        subpass_index,
        &used,
        layouts,
    )?);

    Ok(())
}

/// `EXTERNAL → 0`，以及 `i−1 → i`（与原先 `MainCameraPass` 中链式依赖一致）。
fn build_linear_subpass_dependencies(subpass_count: usize) -> Vec<vk::SubpassDependency> {
    let mut deps = Vec::with_capacity(subpass_count);
    if subpass_count == 0 {
        return deps;
    }

    deps.push(
        vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .build(),
    );

    for i in 1..subpass_count {
        let src = (i - 1) as u32;
        let dst = i as u32;
        deps.push(
            vk::SubpassDependency::builder()
                .src_subpass(src)
                .dst_subpass(dst)
                .src_stage_mask(
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::FRAGMENT_SHADER,
                )
                .dst_stage_mask(
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::FRAGMENT_SHADER,
                )
                .src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        );
    }
    deps
}