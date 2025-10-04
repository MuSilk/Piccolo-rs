enum PrimitiveType {
    Point,
    Line,
    Triangle,
    Quad,
}

struct RawVertexBuffer {
    vertex_count: u32,
    positions: Vec<f32>,
    normals: Vec<f32>,
    tangents: Vec<f32>,
    uvs: Vec<f32>,
}

struct RawIndexBuffer {
    primitive_type: PrimitiveType,
    primitive_count: u32,
    indices: Vec<u32>,
}

struct MaterialTexture {
    base_color: String,
    metalloc_roughness: String,
    normal: String,
}

pub struct StaticMeshData {
    vertex_buffer: RawVertexBuffer,
    index_buffer: RawIndexBuffer,
    material_texture: MaterialTexture,
}