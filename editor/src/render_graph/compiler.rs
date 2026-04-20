use std::collections::{BinaryHeap, HashMap};

use anyhow::Result;
use serde::Serialize;

use super::graph_types::{GraphNode, NodeKind, PortType, RenderGraphAsset, input_ports_for, output_ports_for};
use super::script_lang::{CompiledScript, compile_script};

#[derive(Clone, Debug)]
pub struct CompileDiagnostic {
    pub node_id: Option<u64>,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum CompiledNodeOp {
    Builtin,
    Script { program: CompiledScript },
}

#[derive(Clone, Debug)]
pub struct CompiledNode {
    pub id: u64,
    pub name: String,
    pub kind: NodeKind,
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub incoming: Vec<(String, u64, String)>,
    pub op: CompiledNodeOp,
}

#[derive(Clone, Debug, Default)]
pub struct CompiledNodeGraph {
    pub order: Vec<u64>,
    pub nodes: HashMap<u64, CompiledNode>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PipelinePlanExport {
    pub version: u32,
    pub execution_order: Vec<u64>,
    pub nodes: Vec<PipelinePlanNodeExport>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PipelinePlanNodeExport {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub fn compile_graph(asset: &RenderGraphAsset) -> std::result::Result<CompiledNodeGraph, Vec<CompileDiagnostic>> {
    let mut diags: Vec<CompileDiagnostic> = Vec::new();
    let order = match topo_sort(asset) {
        Ok(v) => v,
        Err(msg) => {
            diags.push(CompileDiagnostic {
                node_id: None,
                message: msg,
            });
            return Err(diags);
        }
    };

    let mut nodes: HashMap<u64, &GraphNode> = HashMap::new();
    for n in &asset.nodes {
        nodes.insert(n.id, n);
    }

    for e in &asset.edges {
        let Some(from) = nodes.get(&e.from_node) else {
            diags.push(CompileDiagnostic {
                node_id: Some(e.from_node),
                message: format!("edge source node not found: {}", e.from_node),
            });
            continue;
        };
        let Some(to) = nodes.get(&e.to_node) else {
            diags.push(CompileDiagnostic {
                node_id: Some(e.to_node),
                message: format!("edge target node not found: {}", e.to_node),
            });
            continue;
        };
        let outs = output_ports_for(from);
        if !outs.iter().any(|p| p.name == e.from_port) {
            diags.push(CompileDiagnostic {
                node_id: Some(from.id),
                message: format!("unknown output port: {}", e.from_port),
            });
        }
        let ins = input_ports_for(to);
        let Some(input_port) = ins.iter().find(|p| p.name == e.to_port) else {
            diags.push(CompileDiagnostic {
                node_id: Some(to.id),
                message: format!("unknown input port: {}", e.to_port),
            });
            continue;
        };
        let out_ty = outs
            .iter()
            .find(|p| p.name == e.from_port)
            .map(|p| p.ty)
            .unwrap_or(PortType::SceneColor);
        if out_ty != input_port.ty {
            diags.push(CompileDiagnostic {
                node_id: Some(to.id),
                message: format!(
                    "port type mismatch: {}.{} -> {}.{}",
                    e.from_node, e.from_port, e.to_node, e.to_port
                ),
            });
        }
    }

    let mut compiled = CompiledNodeGraph::default();
    compiled.order = order.clone();
    for node_id in order {
        let Some(node) = nodes.get(&node_id) else {
            continue;
        };
        let input_ports = input_ports_for(node).into_iter().map(|p| p.name).collect::<Vec<_>>();
        let output_ports = output_ports_for(node).into_iter().map(|p| p.name).collect::<Vec<_>>();
        let incoming = asset
            .edges
            .iter()
            .filter(|e| e.to_node == node.id)
            .map(|e| (e.to_port.clone(), e.from_node, e.from_port.clone()))
            .collect::<Vec<_>>();
        let op = if matches!(node.kind, NodeKind::ScriptNode) {
            if node.script.is_none() {
                diags.push(CompileDiagnostic {
                    node_id: Some(node.id),
                    message: "ScriptNode missing `script` field".to_string(),
                });
            }
            if let Some(script_spec) = node.script.as_ref() {
                match compile_script(&script_spec.source, &input_ports, &output_ports) {
                    Ok(program) => CompiledNodeOp::Script { program },
                    Err(e) => {
                        diags.push(CompileDiagnostic {
                            node_id: Some(node.id),
                            message: format!("script compile failed: {e}"),
                        });
                        CompiledNodeOp::Builtin
                    }
                }
            } else {
                CompiledNodeOp::Builtin
            }
        } else {
            CompiledNodeOp::Builtin
        };
        compiled.nodes.insert(
            node.id,
            CompiledNode {
                id: node.id,
                name: node.name.clone(),
                kind: node.kind.clone(),
                input_ports,
                output_ports,
                incoming,
                op,
            },
        );
    }

    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(compiled)
}

pub fn export_pipeline_plan(compiled: &CompiledNodeGraph) -> PipelinePlanExport {
    let mut nodes = Vec::new();
    for id in &compiled.order {
        if let Some(n) = compiled.nodes.get(id) {
            nodes.push(PipelinePlanNodeExport {
                id: n.id,
                name: n.name.clone(),
                kind: format!("{:?}", n.kind),
                inputs: n.input_ports.clone(),
                outputs: n.output_ports.clone(),
            });
        }
    }
    PipelinePlanExport {
        version: 1,
        execution_order: compiled.order.clone(),
        nodes,
    }
}

pub fn save_pipeline_plan(path: &str, plan: &PipelinePlanExport) -> Result<()> {
    let json = serde_json::to_string_pretty(plan)?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

fn topo_sort(graph: &RenderGraphAsset) -> std::result::Result<Vec<u64>, String> {
    if graph.nodes.is_empty() {
        return Err("render graph has no nodes".to_string());
    }
    let mut node_ids = HashMap::new();
    for n in &graph.nodes {
        if node_ids.insert(n.id, ()).is_some() {
            return Err(format!("duplicate node id {}", n.id));
        }
    }
    let mut indegree: HashMap<u64, usize> = node_ids.keys().map(|id| (*id, 0)).collect();
    let mut out_edges: HashMap<u64, Vec<u64>> = HashMap::new();
    for e in &graph.edges {
        if !node_ids.contains_key(&e.from_node) || !node_ids.contains_key(&e.to_node) {
            return Err("edge references unknown node".to_string());
        }
        out_edges.entry(e.from_node).or_default().push(e.to_node);
        if let Some(v) = indegree.get_mut(&e.to_node) {
            *v += 1;
        }
    }
    let mut ready: BinaryHeap<std::cmp::Reverse<u64>> = indegree
        .iter()
        .filter_map(|(id, deg)| (*deg == 0).then_some(std::cmp::Reverse(*id)))
        .collect();
    let mut order = Vec::with_capacity(graph.nodes.len());
    while let Some(std::cmp::Reverse(n)) = ready.pop() {
        order.push(n);
        if let Some(targets) = out_edges.get(&n) {
            for to in targets {
                if let Some(v) = indegree.get_mut(to) {
                    *v -= 1;
                    if *v == 0 {
                        ready.push(std::cmp::Reverse(*to));
                    }
                }
            }
        }
    }
    if order.len() != graph.nodes.len() {
        return Err("render graph contains cycle".to_string());
    }
    Ok(order)
}
