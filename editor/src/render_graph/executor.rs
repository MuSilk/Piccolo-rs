use std::collections::HashMap;
use std::time::Instant;

use anyhow::{Result, anyhow};

use super::compiler::{CompiledNodeGraph, CompiledNodeOp};
use super::script_lang::{ScriptValue, eval_script};

#[derive(Clone, Debug)]
pub struct NodeExecutionLog {
    pub node_id: u64,
    pub node_name: String,
    pub elapsed_us: u128,
    pub output_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionReport {
    pub logs: Vec<NodeExecutionLog>,
    pub final_values: HashMap<(u64, String), ScriptValue>,
}

pub fn run_graph(compiled: &CompiledNodeGraph) -> Result<ExecutionReport> {
    let mut report = ExecutionReport::default();
    for node_id in &compiled.order {
        let node = compiled
            .nodes
            .get(node_id)
            .ok_or_else(|| anyhow!("compiled node missing: {node_id}"))?;
        let start = Instant::now();
        let mut input_map: HashMap<String, ScriptValue> = HashMap::new();
        for (to_port, from_node, from_port) in &node.incoming {
            if let Some(v) = report.final_values.get(&(*from_node, from_port.clone())) {
                input_map.insert(to_port.clone(), v.clone());
            }
        }
        for p in &node.input_ports {
            input_map
                .entry(p.clone())
                .or_insert(ScriptValue::Scalar(0.0));
        }

        let outputs = match &node.op {
            CompiledNodeOp::Script { program } => eval_script(program, &input_map)?,
            CompiledNodeOp::Builtin => {
                let mut out = HashMap::new();
                let passthrough = input_map
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or(ScriptValue::Scalar(0.0));
                for p in &node.output_ports {
                    out.insert(p.clone(), passthrough.clone());
                }
                out
            }
        };

        for (k, v) in outputs {
            report.final_values.insert((*node_id, k), v);
        }
        report.logs.push(NodeExecutionLog {
            node_id: *node_id,
            node_name: node.name.clone(),
            elapsed_us: start.elapsed().as_micros(),
            output_count: node.output_ports.len(),
        });
    }
    Ok(report)
}
