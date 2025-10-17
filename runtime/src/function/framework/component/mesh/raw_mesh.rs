#[derive(Default)]
enum PrimitiveType {
    Point,
    Line,
    #[default]
    Triangle,
    Quad,
}

#[derive(Default)]
pub struct RawVertexBuffer {
    vertex_count: u32,
    positions: Vec<f32>,
    normals: Vec<f32>,
    tangents: Vec<f32>,
    uvs: Vec<f32>,
}

#[derive(Default)]
pub struct RawIndexBuffer {
    primitive_type: PrimitiveType,
    primitive_count: u32,
    indices: Vec<u32>,
}

#[derive(Default)]
pub struct MaterialTexture {
    base_color: String,
    metalloc_roughness: String,
    normal: String,
}

#[derive(Default)]
pub struct StaticMeshData {
    pub vertex_buffer: RawVertexBuffer,
    pub index_buffer: RawIndexBuffer,
    pub material_texture: MaterialTexture,
}