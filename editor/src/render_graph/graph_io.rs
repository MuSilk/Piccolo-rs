use std::{fs, io, path::Path};

use super::graph_types::RenderGraphAsset;

pub const DEFAULT_GRAPH_PATH: &str = "asset/render_graph/default_graph.json";
pub const MAIN_CAMERA_PASS_GRAPH_PATH: &str = "asset/render_pipeline/main_camera_pass.json";

pub fn save_graph_to_file(graph: &RenderGraphAsset, path: &str) -> io::Result<()> {
    let file_path = Path::new(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(graph)
        .map_err(|e| io::Error::other(format!("serialize graph failed: {e}")))?;
    fs::write(file_path, content)
}

pub fn load_graph_from_file(path: &str) -> io::Result<RenderGraphAsset> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str::<RenderGraphAsset>(&content)
        .map_err(|e| io::Error::other(format!("parse graph failed: {e}")))
}
