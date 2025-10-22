
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub px: f32, pub py: f32, pub pz: f32,
    pub nx: f32, pub ny: f32, pub nz: f32,
    pub tx: f32, pub ty: f32, pub tz: f32,
    pub u: f32,  pub v: f32,
}

#[derive(Debug ,Default, serde::Serialize, serde::Deserialize)]
pub struct SkeletonBinding {
    index0: u32, index1: u32, index2: u32, index3: u32,
    weight0: f32, weight1: f32, weight2: f32, weight3: f32,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bindings: Vec<SkeletonBinding>,
}