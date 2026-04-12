//! 体素世界：固定范围网格 + 解析高度场 + 面剔除网格；与 `minecraft::world` 等实现无关。

use std::{cell::RefCell, rc::Rc};

use serde::Deserialize;
use runtime::{
    core::math::{axis_aligned::AxisAlignedBox, matrix4::Matrix4x4, vector3::Vector3},
    engine::Engine,
    function::{
        framework::{
            component::{component::ComponentTrait, mesh::mesh_component::MeshComponent, transform_component::TransformComponent},
            resource::component::mesh::MeshComponentRes,
            scene::scene::Scene,
        },
        render::{
            render_object::{GameObjectDynamicMeshDesc, GameObjectMeshDesc, GameObjectPartDesc},
            render_type::MeshVertexDataDefinition,
        },
    }
};

/// 与 `block.json` 结构一致，避免注册与 `minecraft::BlockRes` 相同的 typetag 类型。
#[derive(Deserialize)]
struct BlockMeshJson {
    m_mesh_res: MeshComponentRes,
}

pub const WX: i32 = 48;
pub const WY: i32 = 48;
pub const WZ: i32 = 96;

const ATLAS: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
}

impl VoxelKind {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grass,
            2 => Self::Dirt,
            3 => Self::Stone,
            _ => Self::Air,
        }
    }
}

fn cell_index(x: i32, y: i32, z: i32) -> Option<usize> {
    if x < 0 || y < 0 || z < 0 || x >= WX || y >= WY || z >= WZ {
        return None;
    }
    Some((x + WX * (y + WY * z)) as usize)
}

fn surface_height(wx: i32, wy: i32) -> i32 {
    let fx = wx as f32 * 0.11;
    let fy = wy as f32 * 0.13;
    let h = 18.0 + 16.0 * (fx.sin() + 0.6 * fy.cos()).clamp(-1.2, 1.2);
    h.clamp(8.0, (WZ - 4) as f32) as i32
}

/// `face` 与下方 `neighbors` 一致：0..3 为竖直面，4=+Z 顶，5=-Z 底。
fn atlas_cell(kind: VoxelKind, face: usize) -> (u32, u32) {
    match kind {
        VoxelKind::Grass => match face {
            4 => (1, 0), // +Z 顶面
            5 => (1, 0), // -Z 底面（与主工程草方块一致）
            _ => (0, 0), // 四个侧面
        },
        VoxelKind::Dirt => (2, 0),
        VoxelKind::Stone => (3, 0),
        VoxelKind::Air => (0, 0),
    }
}

fn push_quad(
    verts: &mut Vec<MeshVertexDataDefinition>,
    idx: &mut Vec<u32>,
    corners: [MeshVertexDataDefinition; 4],
) {
    let base = verts.len() as u32;
    verts.extend_from_slice(&corners);
    idx.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn apply_tile(v: &mut MeshVertexDataDefinition, cell: (u32, u32)) {
    v.u = (v.u + cell.0 as f32) / ATLAS;
    v.v = (v.v + cell.1 as f32) / ATLAS;
}

fn transform_corner(v: &mut MeshVertexDataDefinition, origin: Vector3, m: &Matrix4x4) {
    let p = Vector3::new(v.x, v.y, v.z);
    let p = *m * p.to_homogeneous();
    v.x = p.x + origin.x;
    v.y = p.y + origin.y;
    v.z = p.z + origin.z;
}

/// 在体素 (ix,iy,iz) 上为朝 +X 的外露面生成四顶点（局部 0..1 立方体再平移）。
fn emit_faces_for_cell(
    ix: i32,
    iy: i32,
    iz: i32,
    kind: VoxelKind,
    get: impl Fn(i32, i32, i32) -> VoxelKind,
    verts: &mut Vec<MeshVertexDataDefinition>,
    idx: &mut Vec<u32>,
) {
    let o = Vector3::new(ix as f32, iy as f32, iz as f32);
    let neighbors = [
        (1i32, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    for (fi, &(dx, dy, dz)) in neighbors.iter().enumerate() {
        let nk = get(ix + dx, iy + dy, iz + dz);
        if nk != VoxelKind::Air {
            continue;
        }
        let cell = atlas_cell(kind, fi);
        let mut tr = |local: [MeshVertexDataDefinition; 4]| {
            let mut out = local;
            let mat = Matrix4x4::identity();
            for v in &mut out {
                transform_corner(v, o, &mat);
                apply_tile(v, cell);
            }
            push_quad(verts, idx, out);
        };
        match fi {
            0 => {
                // +X
                tr([
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                        nx: 1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: 1.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                        nx: 1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: 1.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        nx: 1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: 1.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                        nx: 1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: 1.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                ]);
            }
            1 => {
                // -X
                tr([
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                        nx: -1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: -1.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        nx: -1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: -1.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                        nx: -1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: -1.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 1.0,
                        nx: -1.0,
                        ny: 0.0,
                        nz: 0.0,
                        tx: 0.0,
                        ty: -1.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                ]);
            }
            2 => {
                // +Y
                tr([
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 1.0,
                        nz: 0.0,
                        tx: -1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 1.0,
                        nz: 0.0,
                        tx: -1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 1.0,
                        nz: 0.0,
                        tx: -1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 1.0,
                        nz: 0.0,
                        tx: -1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                ]);
            }
            3 => {
                // -Y
                tr([
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: -1.0,
                        nz: 0.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: -1.0,
                        nz: 0.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: -1.0,
                        nz: 0.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: -1.0,
                        nz: 0.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                ]);
            }
            4 => {
                // +Z top
                tr([
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: 1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: 1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: 1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: 1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                ]);
            }
            _ => {
                // -Z bottom
                tr([
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: -1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: -1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 0.0,
                    },
                    MeshVertexDataDefinition {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: -1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 1.0,
                        v: 1.0,
                    },
                    MeshVertexDataDefinition {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                        nx: 0.0,
                        ny: 0.0,
                        nz: -1.0,
                        tx: 1.0,
                        ty: 0.0,
                        tz: 0.0,
                        u: 0.0,
                        v: 1.0,
                    },
                ]);
            }
        }
    }
}

fn build_mesh(cells: &[u8]) -> GameObjectDynamicMeshDesc {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let get = |x: i32, y: i32, z: i32| -> VoxelKind {
        cell_index(x, y, z)
            .map(|i| VoxelKind::from_u8(cells[i]))
            .unwrap_or(VoxelKind::Air)
    };
    for z in 0..WZ {
        for y in 0..WY {
            for x in 0..WX {
                let k = get(x, y, z);
                if k == VoxelKind::Air {
                    continue;
                }
                emit_faces_for_cell(x, y, z, k, &get, &mut verts, &mut indices);
            }
        }
    }
    let mut mesh = GameObjectDynamicMeshDesc::default();
    mesh.m_is_dirty = true;
    mesh.m_mesh_file = "minecraft_ai_voxel_world.mesh".to_string();
    mesh.m_vertices = verts;
    mesh.m_indices = indices;
    mesh
}

pub struct VoxelWorld {
    cells: Vec<u8>,
    pub mesh: Rc<RefCell<GameObjectDynamicMeshDesc>>,
}

impl VoxelWorld {
    pub fn new_box(engine: &Engine, scene: &mut Scene) -> Box<Self> {
        let n = (WX * WY * WZ) as usize;
        let mut cells = vec![0u8; n];
        for y in 0..WY {
            for x in 0..WX {
                let top = surface_height(x, y);
                for z in 0..top {
                    let t = if z == top - 1 {
                        VoxelKind::Grass
                    } else if z >= top - 3 {
                        VoxelKind::Dirt
                    } else {
                        VoxelKind::Stone
                    };
                    if let Some(i) = cell_index(x, y, z) {
                        cells[i] = t as u8;
                    }
                }
            }
        }
        let mesh = Rc::new(RefCell::new(build_mesh(&cells)));
        let object_id = scene.spawn();
        let asset_manager = engine.asset_manager();
        let config_manager = engine.config_manager();
        let mesh_res = asset_manager
            .load_asset::<BlockMeshJson>(config_manager, "asset/minecraft-ai/block.json")
            .expect("asset/minecraft-ai/block.json")
            .m_mesh_res;
        let mut mesh_component = Box::new(MeshComponent::default());
        mesh_component.post_load_resource(object_id, asset_manager, config_manager, &mesh_res);
        mesh_component.m_raw_meshes.resize(1, GameObjectPartDesc::default());
        mesh_component.m_raw_meshes[0].m_mesh_desc = GameObjectMeshDesc::DynamicMesh(Rc::clone(&mesh));
        let mut transform = Box::new(TransformComponent::default());
        transform.post_load_resource(runtime::core::math::transform::Transform::default());
        let components = vec![
            RefCell::new(mesh_component) as RefCell<Box<dyn ComponentTrait>>,
            RefCell::new(transform),
        ];
        scene.create_object(object_id, components);
        Box::new(Self { cells, mesh })
    }

    pub fn suggested_spawn() -> Vector3 {
        let x = WX as f32 * 0.5;
        let y = WY as f32 * 0.5;
        let z = surface_height((x as i32).clamp(0, WX - 1), (y as i32).clamp(0, WY - 1)) as f32 + 4.0;
        Vector3::new(x, y, z)
    }

    pub fn collect_block_hits(&self, area: &AxisAlignedBox) -> Vec<AxisAlignedBox> {
        let mut out = Vec::new();
        let x0 = area.get_min_corner().x.floor() as i32;
        let x1 = area.get_max_corner().x.floor() as i32;
        let y0 = area.get_min_corner().y.floor() as i32;
        let y1 = area.get_max_corner().y.floor() as i32;
        let z0 = area.get_min_corner().z.floor() as i32;
        let z1 = area.get_max_corner().z.floor() as i32;
        for ix in x0..=x1 {
            for iy in y0..=y1 {
                for iz in z0..=z1 {
                    let Some(i) = cell_index(ix, iy, iz) else {
                        continue;
                    };
                    if self.cells[i] == 0 {
                        continue;
                    }
                    out.push(AxisAlignedBox::new(
                        Vector3::new(ix as f32 + 0.5, iy as f32 + 0.5, iz as f32 + 0.5),
                        Vector3::new(0.5, 0.5, 0.5),
                    ));
                }
            }
        }
        out
    }
}
