use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use anyhow::{Result, anyhow, bail};
use serde::Deserialize;

use crate::function::render::interface::vulkan::vulkan_rhi::VulkanRHI;
use crate::function::render::render_graph::{
    CompiledRenderGraph, RenderGraphAsset, RenderGraphEdge, RenderGraphNode, RenderGraphNodeKind,
    resolve_shader_subpass_inputs,
};
use crate::function::render::passes::main_camera_pass::{
    _MAIN_CAMERA_PASS_ATTACHMENT_COUNT, _MAIN_CAMERA_PASS_BACKUP_BUFFER_EVEN, _MAIN_CAMERA_PASS_BACKUP_BUFFER_ODD,
    _MAIN_CAMERA_PASS_DEPTH, _MAIN_CAMERA_PASS_GBUFFER_A, _MAIN_CAMERA_PASS_GBUFFER_B, _MAIN_CAMERA_PASS_GBUFFER_C,
    _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_EVEN, _MAIN_CAMERA_PASS_POST_PROCESS_BUFFER_ODD, _MAIN_CAMERA_PASS_SWAPCHAIN_IMAGE,
};
use crate::function::render::render_pass::FrameBufferAttachment;
use crate::resource::config_manager::ConfigManager;
use vulkanalia::{prelude::v1_0::*};

/// 端口级 G-buffer / 延迟光照 / 后处理 / FXAA / UI / Combine 管线（与 `main_camera_pass_deferred_light_fxaa.json` 同 schema）。
pub const MAIN_CAMERA_DEFERRED_LIGHT_FXAA_PIPELINE_GRAPH_RELATIVE_PATH: &str =
    "render_pipeline/main_camera_pass_deferred_light_fxaa.json";

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

// ---------------------------------------------------------------------------
// 端口图（`main_camera_pass_deferred_light_fxaa.json`）：独立推导 attachment + RenderPass
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PortPipelineGraphAsset {
    #[serde(default)]
    pub version: u32,
    pub nodes: Vec<PortPipelineNode>,
    pub edges: Vec<PortPipelineEdge>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortPipelineNode {
    pub id: u64,
    pub name: String,
    #[serde(rename = "vert_spv")]
    pub vert_spv: String,
    #[serde(rename = "frag_spv")]
    pub frag_spv: String,
    #[serde(default, rename = "in_port")]
    pub in_ports: Vec<PortPipelinePort>,
    #[serde(rename = "out_port")]
    pub out_ports: Vec<PortPipelinePort>,
    #[serde(default)]
    pub position: [f32; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortPipelinePort {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: PortPipelinePortKind,
    #[serde(default)]
    pub usage: Option<String>,
    pub format: String,
    pub width: String,
    pub height: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PortPipelinePortKind {
    ColorAttachment,
    DepthAttachment,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortPipelineEdge {
    pub from_node: u64,
    pub from_port: String,
    pub to_node: u64,
    pub to_port: String,
}

#[derive(Debug, Default)]
pub struct CompiledPortPipelineGraph {
    pub execution_order: Vec<u64>,
}

/// `build_render_pass` 的产物：`VkRenderPass` 与按 attachment 索引对齐的 framebuffer 槽位（swapchain 槽为 `None`）。
pub struct BuiltPortGraphPass {
    pub render_pass: vk::RenderPass,
    pub framebuffer_slots: Vec<Option<FrameBufferAttachment>>,
    pub extent: vk::Extent2D,
}

pub fn load_port_pipeline_graph_from_relative(
    config_manager: &ConfigManager,
    relative_to_asset_folder: &str,
) -> Result<PortPipelineGraphAsset> {
    let file_path = config_manager.get_asset_folder().join(relative_to_asset_folder);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| anyhow!("read port pipeline graph failed ({}): {e}", file_path.display()))?;
    serde_json::from_str::<PortPipelineGraphAsset>(&content)
        .map_err(|e| anyhow!("parse port pipeline graph failed ({}): {e}", file_path.display()))
}

pub fn compile_port_pipeline_graph(graph: &PortPipelineGraphAsset) -> Result<CompiledPortPipelineGraph> {
    if graph.nodes.is_empty() {
        bail!("port pipeline graph has no nodes");
    }
    let mut node_ids: HashMap<u64, ()> = HashMap::new();
    for n in &graph.nodes {
        if node_ids.insert(n.id, ()).is_some() {
            bail!("duplicate node id {}", n.id);
        }
    }
    let mut indegree: HashMap<u64, usize> = node_ids.keys().map(|id| (*id, 0)).collect();
    let mut out_edges: HashMap<u64, Vec<u64>> = HashMap::new();
    for e in &graph.edges {
        if !node_ids.contains_key(&e.from_node) {
            bail!("edge has unknown from_node {}", e.from_node);
        }
        if !node_ids.contains_key(&e.to_node) {
            bail!("edge has unknown to_node {}", e.to_node);
        }
        out_edges.entry(e.from_node).or_default().push(e.to_node);
        if let Some(x) = indegree.get_mut(&e.to_node) {
            *x += 1;
        }
    }
    let mut ready: BinaryHeap<Reverse<u64>> = indegree
        .iter()
        .filter_map(|(id, deg)| (*deg == 0).then_some(Reverse(*id)))
        .collect();
    let mut order: Vec<u64> = Vec::with_capacity(node_ids.len());
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
    if order.len() != node_ids.len() {
        bail!("port pipeline graph contains cycle");
    }
    Ok(CompiledPortPipelineGraph {
        execution_order: order,
    })
}

fn port_format_from_token(s: &str, rhi: &VulkanRHI) -> Result<vk::Format> {
    let t = s.trim();
    if t == "$swapchainImageFormat$" {
        return Ok(rhi.get_swapchain_info().image_format);
    }
    if t == "$depthImageFormat$" {
        return Ok(rhi.get_depth_image_info().format);
    }
    port_parse_static_format(t)
}

fn port_parse_static_format(t: &str) -> Result<vk::Format> {
    Ok(match t {
        "R8G8B8A8_SNORM" => vk::Format::R8G8B8A8_SNORM,
        "R8G8B8A8_SRGB" => vk::Format::R8G8B8A8_SRGB,
        "R16G16B16A16_SFLOAT" => vk::Format::R16G16B16A16_SFLOAT,
        _ => bail!("unknown format token: {t:?}"),
    })
}

fn port_resolve_dim_token(s: &str, extent: vk::Extent2D) -> Result<u32> {
    match s.trim() {
        "$swapchainImageWidth$" => Ok(extent.width),
        "$swapchainImageHeight$" => Ok(extent.height),
        x => x
            .parse::<u32>()
            .map_err(|e| anyhow!("invalid dimension {x:?}: {e}")),
    }
}

fn port_depth_image_aspect(format: vk::Format) -> vk::ImageAspectFlags {
    match format {
        vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        }
        _ => vk::ImageAspectFlags::DEPTH,
    }
}

fn port_node_by_id<'a>(asset: &'a PortPipelineGraphAsset, id: u64) -> Result<&'a PortPipelineNode> {
    asset
        .nodes
        .iter()
        .find(|n| n.id == id)
        .ok_or_else(|| anyhow!("unknown node id {id}"))
}

fn port_ensure_output(node: &PortPipelineNode, port: &str) -> Result<()> {
    if node.out_ports.iter().any(|p| p.name == port) {
        return Ok(());
    }
    bail!(
        "node {} ({}) has no output port {:?}",
        node.id,
        node.name,
        port
    );
}

fn port_collect_input_names(node: &PortPipelineNode, edges: &[PortPipelineEdge]) -> Vec<String> {
    let mut names: Vec<String> = node.in_ports.iter().map(|p| p.name.clone()).collect();
    let known: HashSet<String> = names.iter().cloned().collect();
    let mut extra: Vec<String> = edges
        .iter()
        .filter(|e| e.to_node == node.id && !known.contains(&e.to_port))
        .map(|e| e.to_port.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    extra.sort_unstable();
    names.extend(extra);
    names
}

#[derive(Clone)]
struct PortSubpassLayout {
    color_out_ports: Vec<String>,
    input_ports: Vec<String>,
    depth_out_port: Option<String>,
}

fn port_subpass_layout(node: &PortPipelineNode, edges: &[PortPipelineEdge]) -> PortSubpassLayout {
    let color_out_ports = node
        .out_ports
        .iter()
        .filter(|p| p.kind == PortPipelinePortKind::ColorAttachment)
        .map(|p| p.name.clone())
        .collect();
    let depth_out_port = node
        .out_ports
        .iter()
        .find(|p| p.kind == PortPipelinePortKind::DepthAttachment)
        .map(|p| p.name.clone());
    let input_ports = port_collect_input_names(node, edges);
    PortSubpassLayout {
        color_out_ports,
        input_ports,
        depth_out_port,
    }
}

fn port_producer_attachment_indices(
    asset: &PortPipelineGraphAsset,
    order: &[u64],
    rhi: &VulkanRHI,
) -> Result<(HashMap<(u64, String), usize>, Vec<PortAttachmentBuildInfo>)> {
    let mut key_to_index: HashMap<(u64, String), usize> = HashMap::new();
    let mut list: Vec<PortAttachmentBuildInfo> = Vec::new();
    for &nid in order {
        let node = port_node_by_id(asset, nid)?;
        for p in &node.out_ports {
            if key_to_index.insert((nid, p.name.clone()), list.len()).is_some() {
                bail!("duplicate output port {} on node {}", p.name, nid);
            }
            let format = port_format_from_token(&p.format, rhi)?;
            let is_swapchain = p.format.trim() == "$swapchainImageFormat$";
            let is_depth = p.kind == PortPipelinePortKind::DepthAttachment;
            list.push(PortAttachmentBuildInfo {
                format,
                is_depth,
                is_swapchain,
                needs_sampled: false,
                needs_input_attachment: false,
                producer_node: nid,
                producer_port: p.name.clone(),
            });
        }
    }
    Ok((key_to_index, list))
}

#[derive(Clone, Debug)]
struct PortAttachmentBuildInfo {
    format: vk::Format,
    is_depth: bool,
    is_swapchain: bool,
    needs_sampled: bool,
    needs_input_attachment: bool,
    producer_node: u64,
    producer_port: String,
}

fn port_apply_consumer_usage_flags(
    asset: &PortPipelineGraphAsset,
    build: &mut [PortAttachmentBuildInfo],
    key_to_index: &HashMap<(u64, String), usize>,
) -> Result<()> {
    for e in &asset.edges {
        port_ensure_output(port_node_by_id(asset, e.from_node)?, &e.from_port)?;
        let prod_i = *key_to_index
            .get(&(e.from_node, e.from_port.clone()))
            .ok_or_else(|| anyhow!("internal: missing producer index for edge {:?}", e))?;
        let consumer = port_node_by_id(asset, e.to_node)?;
        let usage = consumer
            .in_ports
            .iter()
            .find(|p| p.name == e.to_port)
            .and_then(|p| p.usage.as_deref());
        let u = usage.unwrap_or("subpassInput");
        match u {
            "subpassInput" => build[prod_i].needs_input_attachment = true,
            "sampler2D" => {
                build[prod_i].needs_sampled = true;
                build[prod_i].needs_input_attachment = true;
            }
            _ => bail!(
                "node {} port {:?}: unknown usage {:?}",
                e.to_node,
                e.to_port,
                usage
            ),
        }
    }
    Ok(())
}

fn port_attachment_ref_color_out(
    key_to_index: &HashMap<(u64, String), usize>,
    from_node: u64,
    from_port: &str,
) -> Result<vk::AttachmentReference> {
    let idx = *key_to_index
        .get(&(from_node, from_port.to_string()))
        .ok_or_else(|| anyhow!("no attachment for node {from_node} output port {from_port:?}"))?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    })
}

fn port_attachment_ref_input_read(
    edges: &[PortPipelineEdge],
    key_to_index: &HashMap<(u64, String), usize>,
    to_node: u64,
    to_port: &str,
) -> Result<vk::AttachmentReference> {
    let e = edges
        .iter()
        .find(|x| x.to_node == to_node && x.to_port == to_port)
        .ok_or_else(|| anyhow!("no incoming edge to node {to_node} port {to_port:?}"))?;
    let idx = *key_to_index
        .get(&(e.from_node, e.from_port.clone()))
        .ok_or_else(|| {
            anyhow!(
                "incoming edge to {to_node}/{to_port:?} references missing producer attachment {:?}/{}",
                e.from_node,
                e.from_port
            )
        })?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    })
}

fn port_attachment_ref_depth_stencil_out(
    key_to_index: &HashMap<(u64, String), usize>,
    from_node: u64,
    from_port: &str,
) -> Result<vk::AttachmentReference> {
    let idx = *key_to_index
        .get(&(from_node, from_port.to_string()))
        .ok_or_else(|| anyhow!("no depth attachment for node {from_node} port {from_port:?}"))?;
    Ok(vk::AttachmentReference {
        attachment: idx as u32,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    })
}

fn port_attachment_indices_written_by_subpass(
    node_id: u64,
    layout: &PortSubpassLayout,
    key_to_index: &HashMap<(u64, String), usize>,
) -> Result<HashSet<usize>> {
    let mut out = HashSet::new();
    for port in &layout.color_out_ports {
        out.insert(
            *key_to_index
                .get(&(node_id, port.clone()))
                .ok_or_else(|| anyhow!("missing color out {node_id}/{port}"))?,
        );
    }
    if let Some(port) = &layout.depth_out_port {
        out.insert(
            *key_to_index
                .get(&(node_id, port.clone()))
                .ok_or_else(|| anyhow!("missing depth out {node_id}/{port}"))?,
        );
    }
    Ok(out)
}

fn port_attachment_indices_used_this_subpass(
    edges: &[PortPipelineEdge],
    node_id: u64,
    layout: &PortSubpassLayout,
    key_to_index: &HashMap<(u64, String), usize>,
) -> Result<HashSet<usize>> {
    let mut u = port_attachment_indices_written_by_subpass(node_id, layout, key_to_index)?;
    for port in &layout.input_ports {
        let r = port_attachment_ref_input_read(edges, key_to_index, node_id, port)?;
        u.insert(r.attachment as usize);
    }
    Ok(u)
}

fn port_attachment_indices_written_before_subpass(
    order: &[u64],
    _asset: &PortPipelineGraphAsset,
    layouts: &[PortSubpassLayout],
    key_to_index: &HashMap<(u64, String), usize>,
    subpass_index_exclusive_end: usize,
) -> Result<HashSet<usize>> {
    let mut s = HashSet::new();
    for i in 0..subpass_index_exclusive_end {
        let nid = order[i];
        s.extend(port_attachment_indices_written_by_subpass(
            nid,
            &layouts[i],
            key_to_index,
        )?);
    }
    Ok(s)
}

fn port_attachment_indices_read_after_subpass(
    order: &[u64],
    graph: &PortPipelineGraphAsset,
    key_to_index: &HashMap<(u64, String), usize>,
    subpass_index: usize,
) -> Result<HashSet<usize>> {
    let mut s = HashSet::new();
    for &nid in order.iter().skip(subpass_index + 1) {
        for e in graph.edges.iter().filter(|e| e.to_node == nid) {
            let idx = *key_to_index
                .get(&(e.from_node, e.from_port.clone()))
                .ok_or_else(|| anyhow!("edge references unknown producer {:?}/{}", e.from_node, e.from_port))?;
            s.insert(idx);
        }
    }
    Ok(s)
}

fn port_compute_preserve_attachments(
    order: &[u64],
    asset: &PortPipelineGraphAsset,
    subpass_index: usize,
    used: &HashSet<usize>,
    layouts: &[PortSubpassLayout],
    key_to_index: &HashMap<(u64, String), usize>,
) -> Result<Vec<u32>> {
    let written_before = port_attachment_indices_written_before_subpass(
        order,
        asset,
        layouts,
        key_to_index,
        subpass_index,
    )?;
    let read_after = port_attachment_indices_read_after_subpass(order, asset, key_to_index, subpass_index)?;
    let mut v: Vec<u32> = written_before
        .intersection(&read_after)
        .filter(|idx| !used.contains(idx))
        .map(|&idx| idx as u32)
        .collect();
    v.sort_unstable();
    Ok(v)
}

fn port_build_attachment_descriptions(
    build: &[PortAttachmentBuildInfo],
) -> Result<Vec<vk::AttachmentDescription>> {
    let mut v = Vec::with_capacity(build.len());
    for b in build {
        let (initial, final_layout) = if b.is_swapchain {
            (
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::PRESENT_SRC_KHR,
            )
        } else if b.is_depth {
            (
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            )
        } else {
            (
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            )
        };
        v.push(
            vk::AttachmentDescription::builder()
                .format(b.format)
                .samples(vk::SampleCountFlags::_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(initial)
                .final_layout(final_layout)
                .build(),
        );
    }
    Ok(v)
}

fn port_image_usage(b: &PortAttachmentBuildInfo) -> Result<vk::ImageUsageFlags> {
    if b.is_swapchain {
        bail!("port_image_usage: swapchain slot");
    }
    let mut u = vk::ImageUsageFlags::empty();
    if b.is_depth {
        u |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    } else {
        u |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if b.needs_input_attachment {
        u |= vk::ImageUsageFlags::INPUT_ATTACHMENT;
    }
    if b.needs_sampled {
        u |= vk::ImageUsageFlags::SAMPLED;
    } else {
        // 与 `MainCameraPass::setup_attachments` 一致：仅 subpass 链内复用、无需 shader 采样的 RT 可走 LAZILY_ALLOCATED。
        u |= vk::ImageUsageFlags::TRANSIENT_ATTACHMENT;
    }
    Ok(u)
}

fn port_push_subpass_attachment_refs(
    subpass_index: usize,
    node_id: u64,
    order: &[u64],
    layouts: &[PortSubpassLayout],
    asset: &PortPipelineGraphAsset,
    key_to_index: &HashMap<(u64, String), usize>,
    color_refs: &mut Vec<Vec<vk::AttachmentReference>>,
    input_refs: &mut Vec<Vec<vk::AttachmentReference>>,
    depth_refs: &mut Vec<Option<vk::AttachmentReference>>,
    preserve_refs: &mut Vec<Vec<u32>>,
) -> Result<()> {
    let layout = &layouts[subpass_index];
    let mut colors = Vec::new();
    for port in &layout.color_out_ports {
        colors.push(port_attachment_ref_color_out(key_to_index, node_id, port)?);
    }
    color_refs.push(colors);
    let mut inputs = Vec::new();
    for port in &layout.input_ports {
        inputs.push(port_attachment_ref_input_read(
            &asset.edges,
            key_to_index,
            node_id,
            port,
        )?);
    }
    input_refs.push(inputs);
    let depth = if let Some(port) = &layout.depth_out_port {
        Some(port_attachment_ref_depth_stencil_out(
            key_to_index,
            node_id,
            port,
        )?)
    } else {
        None
    };
    depth_refs.push(depth);
    let used = port_attachment_indices_used_this_subpass(&asset.edges, node_id, layout, key_to_index)?;
    preserve_refs.push(port_compute_preserve_attachments(
        order,
        asset,
        subpass_index,
        &used,
        layouts,
        key_to_index,
    )?);
    Ok(())
}

/// 从端口级管线 JSON（如 `main_camera_pass_deferred_light_fxaa.json`）推导 attachment 槽位、创建离屏图像，并构建完整 `VkRenderPass`。
///
/// - 每个节点的 `out_port` 在拓扑序下依次注册为 render pass attachment（含 depth、swapchain 占位）。
/// - 边约定：`from_port` 必须是源节点的输出端口；`subpassInput` / `sampler2D` 决定 `ImageUsage` 是否含 `INPUT_ATTACHMENT` / `SAMPLED`。
/// - `framebuffer_slots[i]` 与 `pAttachments[i]` 对齐；`None` 表示该索引为 swapchain 图像，由调用方在创建 `VkFramebuffer` 时绑定。
pub fn build_render_pass(
    rhi: &VulkanRHI,
    config_manager: &ConfigManager,
    relative_to_asset_folder: &str,
) -> Result<BuiltPortGraphPass> {
    let asset = load_port_pipeline_graph_from_relative(config_manager, relative_to_asset_folder)?;
    build_render_pass_from_asset(rhi, &asset)
}

pub fn build_render_pass_from_asset(rhi: &VulkanRHI, asset: &PortPipelineGraphAsset) -> Result<BuiltPortGraphPass> {
    let compiled = compile_port_pipeline_graph(asset)?;
    let extent = rhi.get_swapchain_info().extent;
    let (key_to_index, mut build_infos) = port_producer_attachment_indices(asset, &compiled.execution_order, rhi)?;
    port_apply_consumer_usage_flags(asset, &mut build_infos, &key_to_index)?;
    for e in &asset.edges {
        port_ensure_output(port_node_by_id(asset, e.from_node)?, &e.from_port)?;
    }
    let attachment_descs = port_build_attachment_descriptions(&build_infos)?;
    let layouts: Vec<PortSubpassLayout> = compiled
        .execution_order
        .iter()
        .map(|&nid| {
            let node = port_node_by_id(asset, nid)?;
            Ok(port_subpass_layout(node, &asset.edges))
        })
        .collect::<Result<_>>()?;
    let mut color_refs: Vec<Vec<vk::AttachmentReference>> = Vec::new();
    let mut input_refs: Vec<Vec<vk::AttachmentReference>> = Vec::new();
    let mut depth_refs: Vec<Option<vk::AttachmentReference>> = Vec::new();
    let mut preserve_refs: Vec<Vec<u32>> = Vec::new();
    for (i, &nid) in compiled.execution_order.iter().enumerate() {
        port_push_subpass_attachment_refs(
            i,
            nid,
            &compiled.execution_order,
            &layouts,
            asset,
            &key_to_index,
            &mut color_refs,
            &mut input_refs,
            &mut depth_refs,
            &mut preserve_refs,
        )?;
    }
    let mut subpasses: Vec<vk::SubpassDescription> = Vec::with_capacity(compiled.execution_order.len());
    for i in 0..compiled.execution_order.len() {
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
    let render_pass = rhi.create_render_pass(&info)?;
    let mut framebuffer_slots: Vec<Option<FrameBufferAttachment>> = vec![None; build_infos.len()];
    for (i, b) in build_infos.iter().enumerate() {
        if b.is_swapchain {
            continue;
        }
        let node = port_node_by_id(asset, b.producer_node)?;
        let port_def = node
            .out_ports
            .iter()
            .find(|p| p.name == b.producer_port)
            .ok_or_else(|| anyhow!("internal: missing port {} on node {}", b.producer_port, b.producer_node))?;
        let w = port_resolve_dim_token(&port_def.width, extent)?;
        let h = port_resolve_dim_token(&port_def.height, extent)?;
        let usage = port_image_usage(b)?;
        let (image, mem) = rhi.create_image(
            w,
            h,
            b.format,
            vk::ImageTiling::OPTIMAL,
            usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageCreateFlags::empty(),
            1,
            1,
        )?;
        let aspect = if b.is_depth {
            port_depth_image_aspect(b.format)
        } else {
            vk::ImageAspectFlags::COLOR
        };
        let view = rhi.create_image_view(
            image,
            b.format,
            aspect,
            vk::ImageViewType::_2D,
            1,
            1,
        )?;
        framebuffer_slots[i] = Some(FrameBufferAttachment {
            image,
            mem,
            view,
            format: b.format,
        });
    }
    Ok(BuiltPortGraphPass {
        render_pass,
        framebuffer_slots,
        extent,
    })
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
        .execution_order
        .iter()
        .map(|&node_id| {
            let node = asset
                .nodes
                .iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| anyhow!("compiled order references unknown node id {node_id}"))?;
            subpass_layout_for_node(node)
        })
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
#[derive(Clone)]
struct SubpassLayout {
    color_out_ports: Vec<String>,
    input_ports: Vec<String>,
    depth: DepthSlot,
}

#[derive(Clone)]
enum DepthSlot {
    None,
    OutgoingFromPort(String),
    IncomingToPort(String),
}

fn subpass_layout_for_node(node: &RenderGraphNode) -> Result<SubpassLayout> {
    use RenderGraphNodeKind::*;
    let kind = &node.kind;
    Ok(match kind {
        BasePass => SubpassLayout {
            color_out_ports: vec![
                "normal".into(),
                "material".into(),
                "base_color".into(),
            ],
            input_ports: vec![],
            depth: DepthSlot::OutgoingFromPort("depth".into()),
        },
        DeferredLighting => SubpassLayout {
            color_out_ports: vec!["deferred_lit".into()],
            input_ports: vec![
                "normal".into(),
                "material".into(),
                "base_color".into(),
                "depth".into(),
            ],
            depth: DepthSlot::None,
        },
        ForwardLighting => SubpassLayout {
            color_out_ports: vec!["lit_hdr".into()],
            input_ports: vec![],
            depth: DepthSlot::IncomingToPort("depth".into()),
        },
        ToneMapping => SubpassLayout {
            color_out_ports: vec!["tone_mapped".into()],
            input_ports: vec!["lit_hdr".into()],
            depth: DepthSlot::None,
        },
        ColorGrading => SubpassLayout {
            color_out_ports: vec!["graded".into()],
            input_ports: vec!["tone_mapped".into()],
            depth: DepthSlot::None,
        },
        Fxaa => SubpassLayout {
            color_out_ports: vec!["antialiased".into()],
            input_ports: vec!["graded".into()],
            depth: DepthSlot::None,
        },
        UiPass => SubpassLayout {
            color_out_ports: vec!["ui_color".into()],
            input_ports: vec![],
            depth: DepthSlot::None,
        },
        CombineUi => SubpassLayout {
            color_out_ports: vec!["present".into()],
            input_ports: vec!["antialiased".into(), "ui_color".into()],
            depth: DepthSlot::None,
        },
        ShaderFullscreen => {
            let spec = node.shader.as_ref().ok_or_else(|| {
                anyhow!(
                    "ShaderFullscreen node {} ({}) requires `shader` field",
                    node.id,
                    node.name
                )
            })?;
            if spec.color_outputs.is_empty() {
                bail!(
                    "ShaderFullscreen node {} ({}): shader.color_outputs must be non-empty",
                    node.id,
                    node.name
                );
            }
            let input_ports = resolve_shader_subpass_inputs(spec)
                .map_err(|e| anyhow!("ShaderFullscreen node {} ({}): {e}", node.id, node.name))?
                .into_iter()
                .map(|r| r.name)
                .collect();
            SubpassLayout {
                color_out_ports: spec.color_outputs.clone(),
                input_ports,
                depth: DepthSlot::None,
            }
        }
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
            &layouts[i],
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
    layout: &SubpassLayout,
) -> Result<HashSet<usize>> {
    let mut out = HashSet::new();
    for port in &layout.color_out_ports {
        out.insert(color_out_attachment_index(edges, node_id, port)?);
    }
    if let DepthSlot::OutgoingFromPort(port) = &layout.depth {
        let e = find_edge_from(edges, node_id, port.as_str()).ok_or_else(|| {
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
    layout: &SubpassLayout,
) -> Result<HashSet<usize>> {
    let mut u = attachment_indices_written_by_subpass(edges, node_id, layout)?;
    for port in &layout.input_ports {
        let e = find_edge_to(edges, node_id, port.as_str()).ok_or_else(|| {
            anyhow!("no incoming edge to node {} port {:?}", node_id, port)
        })?;
        u.insert(framebuffer_name_to_index(&e.framebuffer)?);
    }
    if let DepthSlot::IncomingToPort(port) = &layout.depth {
        let e = find_edge_to(edges, node_id, port.as_str()).ok_or_else(|| {
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
    let layout = &layouts[subpass_index];

    let mut colors = Vec::new();
    for port in &layout.color_out_ports {
        colors.push(attachment_ref_color_out(edges, node_id, port.as_str())?);
    }
    color_refs.push(colors);

    let mut inputs = Vec::new();
    for port in &layout.input_ports {
        inputs.push(attachment_ref_input_read(edges, node_id, port.as_str())?);
    }
    input_refs.push(inputs);

    let depth = match &layout.depth {
        DepthSlot::None => None,
        DepthSlot::OutgoingFromPort(port) => Some(attachment_ref_depth_stencil_out_port(
            edges, node_id, port.as_str(),
        )?),
        DepthSlot::IncomingToPort(port) => Some(attachment_ref_depth_stencil_in_port(
            edges, node_id, port.as_str(),
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

/// 供编辑器与导出工具获取与 `build_subpass` 一致的端口列表（不涉及 Vulkan 对象）。
#[derive(Clone, Debug)]
pub struct SubpassPortLayout {
    pub color_out_ports: Vec<String>,
    pub input_ports: Vec<String>,
}

pub fn subpass_port_layout(node: &RenderGraphNode) -> Result<SubpassPortLayout> {
    let l = subpass_layout_for_node(node)?;
    Ok(SubpassPortLayout {
        color_out_ports: l.color_out_ports,
        input_ports: l.input_ports,
    })
}