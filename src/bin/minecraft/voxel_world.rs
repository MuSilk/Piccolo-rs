//! 体素世界：程序化高度场 + 以玩家为中心的环形区块网格（单动态网格合并），碰撞对任意世界坐标采样，地形在水平方向可无限延伸。
//!
//! 资源使用 `editor/asset/minecraft/block.json`（运行时路径 `asset/minecraft/block.json`）。
//!
//! 区块切换时网格合并 `build_world_mesh` 计算量大；在后台线程构建、主线程每帧取回结果，避免逻辑帧长时间卡住。

use std::{cell::RefCell, collections::HashMap, rc::Rc, thread::JoinHandle};

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

/// 竖直方向体素层数（与高度场上限一致）。
pub const WZ: i32 = 96;
/// 水平向区块边长（体素），与常见 MC 区块宽度一致便于理解。
pub const CHUNK_SIZE: i32 = 16;
/// 以玩家所在区块为中心，Chebyshev 距离 `<=` 该值内的区块参与网格生成（含边界）。
const VIEW_RADIUS_CHUNKS: i32 = 3;

const ATLAS: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
}

fn world_chunk_coords(wx: i32, wy: i32) -> (i32, i32) {
    (wx.div_euclid(CHUNK_SIZE), wy.div_euclid(CHUNK_SIZE))
}

/// 格点伪随机 \([0,1)\)，无三角函数，地形在大尺度上不再呈简单周期。
fn lattice_noise01(ix: i32, iy: i32) -> f32 {
    let mut n = (ix as u32)
        .wrapping_mul(0x9E37_79B1)
        ^ (iy as u32).wrapping_mul(0x85EB_CA6B);
    n = n.wrapping_mul(n | 1);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    (n as f32) * (1.0 / u32::MAX as f32)
}

fn smoothstep3(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 连续 2D 值噪声 \([0,1]\)。
fn value_noise_2d(x: f32, y: f32) -> f32 {
    let x0f = x.floor();
    let y0f = y.floor();
    let x0 = x0f as i32;
    let y0 = y0f as i32;
    let tx = smoothstep3(x - x0f);
    let ty = smoothstep3(y - y0f);
    let n00 = lattice_noise01(x0, y0);
    let n10 = lattice_noise01(x0 + 1, y0);
    let n01 = lattice_noise01(x0, y0 + 1);
    let n11 = lattice_noise01(x0 + 1, y0 + 1);
    let ix0 = n00 + (n10 - n00) * tx;
    let ix1 = n01 + (n11 - n01) * tx;
    ix0 + (ix1 - ix0) * ty
}

/// 多倍频叠加，近似 `-1..1`，再映射到体素层高。
fn surface_height(wx: i32, wy: i32) -> i32 {
    let x = wx as f32;
    let y = wy as f32;
    let mut acc = 0.0_f32;
    let mut wsum = 0.0_f32;
    let mut freq = 0.034_f32;
    let mut w = 1.0_f32;
    for _ in 0..5 {
        let n = value_noise_2d(x * freq, y * freq * 1.09);
        acc += (n - 0.5) * 2.0 * w;
        wsum += w;
        freq *= 2.08;
        w *= 0.5;
    }
    let n = (acc / wsum).clamp(-1.0, 1.0);
    let h = 22.0 + 20.0 * n;
    h.clamp(8.0, (WZ - 4) as f32) as i32
}

fn procedural_voxel(wx: i32, wy: i32, wz: i32) -> VoxelKind {
    if wz < 0 || wz >= WZ {
        return VoxelKind::Air;
    }
    let top = surface_height(wx, wy);
    if wz >= top {
        return VoxelKind::Air;
    }
    if wz == top - 1 {
        VoxelKind::Grass
    } else if wz >= top - 3 {
        VoxelKind::Dirt
    } else {
        VoxelKind::Stone
    }
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

/// 在世界体素坐标 `(wx,wy,wz)` 上为外露面生成四顶点（局部 0..1 立方体再平移）。
fn emit_faces_for_cell(
    wx: i32,
    wy: i32,
    wz: i32,
    kind: VoxelKind,
    get: impl Fn(i32, i32, i32) -> VoxelKind,
    verts: &mut Vec<MeshVertexDataDefinition>,
    idx: &mut Vec<u32>,
) {
    let o = Vector3::new(wx as f32, wy as f32, wz as f32);
    let neighbors = [
        (1i32, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    for (fi, &(dx, dy, dz)) in neighbors.iter().enumerate() {
        let nk = get(wx + dx, wy + dy, wz + dz);
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

fn build_world_mesh(
    center_cx: i32,
    center_cy: i32,
    radius: i32,
    overrides: HashMap<(i32, i32, i32), VoxelKind>,
) -> GameObjectDynamicMeshDesc {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let get = move |x: i32, y: i32, z: i32| {
        if let Some(&k) = overrides.get(&(x, y, z)) {
            k
        } else {
            procedural_voxel(x, y, z)
        }
    };
    let span = CHUNK_SIZE * (radius * 2 + 1);
    let x0 = (center_cx - radius) * CHUNK_SIZE;
    let y0 = (center_cy - radius) * CHUNK_SIZE;
    for lz in 0..WZ {
        for ly in 0..span {
            for lx in 0..span {
                let wx = x0 + lx;
                let wy = y0 + ly;
                let k = get(wx, wy, lz);
                if k == VoxelKind::Air {
                    continue;
                }
                emit_faces_for_cell(wx, wy, lz, k, &get, &mut verts, &mut indices);
            }
        }
    }
    let mut mesh = GameObjectDynamicMeshDesc::default();
    mesh.m_is_dirty = true;
    mesh.m_mesh_file = "minecraft_streaming_voxel_world.mesh".to_string();
    mesh.m_vertices = verts;
    mesh.m_indices = indices;
    mesh
}

pub struct VoxelWorld {
    pub mesh: Rc<RefCell<GameObjectDynamicMeshDesc>>,
    /// 相对程序化地形的体素覆盖（破坏/放置）；与程序化一致时可 `remove` 节省内存。
    overrides: HashMap<(i32, i32, i32), VoxelKind>,
    /// 同区块内编辑后需要刷新合并网格。
    mesh_rebuild_queued: bool,
    /// 当前已提交到 `mesh` 的区块中心（与碰撞程序化采样无关，仅用于流式刷新判定）。
    streaming_center_chunk: Option<(i32, i32)>,
    /// 正在后台构建的网格对应的中心区块（`JoinHandle` 存在时有效）。
    pending_center_chunk: Option<(i32, i32)>,
    mesh_build_join: Option<JoinHandle<GameObjectDynamicMeshDesc>>,
}

impl VoxelWorld {
    pub fn new_box(engine: &Engine, scene: &mut Scene) -> Box<Self> {
        let spawn_xy = (8i32, 8i32);
        let (icx, icy) = world_chunk_coords(spawn_xy.0, spawn_xy.1);
        let mesh = Rc::new(RefCell::new(build_world_mesh(
            icx,
            icy,
            VIEW_RADIUS_CHUNKS,
            HashMap::new(),
        )));
        let object_id = scene.spawn();
        let asset_manager = engine.asset_manager();
        let config_manager = engine.config_manager();
        let mesh_res = asset_manager
            .load_asset::<BlockMeshJson>(config_manager, "asset/minecraft/block.json")
            .expect("asset/minecraft/block.json")
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
        Box::new(Self {
            mesh,
            overrides: HashMap::new(),
            mesh_rebuild_queued: false,
            streaming_center_chunk: Some((icx, icy)),
            pending_center_chunk: None,
            mesh_build_join: None,
        })
    }

    #[inline]
    pub fn voxel_at(&self, wx: i32, wy: i32, wz: i32) -> VoxelKind {
        if let Some(&k) = self.overrides.get(&(wx, wy, wz)) {
            k
        } else {
            procedural_voxel(wx, wy, wz)
        }
    }

    pub fn set_voxel(&mut self, wx: i32, wy: i32, wz: i32, kind: VoxelKind) {
        if kind == procedural_voxel(wx, wy, wz) {
            self.overrides.remove(&(wx, wy, wz));
        } else {
            self.overrides.insert((wx, wy, wz), kind);
        }
        self.mesh_rebuild_queued = true;
    }

    /// 将 `set_voxel` 累积的修改立刻反映到合并网格（主线程同步构建）。
    ///
    /// 破坏/放置若仍走后台线程，要等整幅网格算完才有画面，体感延迟大；编辑后应优先调用本函数。
    /// 若仍有未完成的异步区块构建，会先 `join` 丢弃其结果，再以当前 `overrides` 与玩家所在区块中心重建。
    pub fn flush_voxel_mesh_sync(&mut self, player_position: &Vector3) {
        if !self.mesh_rebuild_queued {
            return;
        }
        self.mesh_rebuild_queued = false;

        if let Some(handle) = self.mesh_build_join.take() {
            let _ = handle.join();
        }
        self.pending_center_chunk = None;

        let wx = player_position.x.floor() as i32;
        let wy = player_position.y.floor() as i32;
        let (cx, cy) = world_chunk_coords(wx, wy);
        let mesh = build_world_mesh(cx, cy, VIEW_RADIUS_CHUNKS, self.overrides.clone());
        *self.mesh.borrow_mut() = mesh;
        self.streaming_center_chunk = Some((cx, cy));
    }

    /// 射线命中第一个非空气体素（世界格坐标），用于破坏。
    pub fn raycast_first_solid(
        &self,
        origin: Vector3,
        dir: Vector3,
        max_dist: f32,
    ) -> Option<(i32, i32, i32)> {
        let dir_len = dir.length();
        if dir_len < 1e-6 {
            return None;
        }
        let dir = dir * (1.0 / dir_len);
        let mut t = 0.08_f32;
        let step = 0.07_f32;
        while t < max_dist {
            let p = origin + dir * t;
            let wx = p.x.floor() as i32;
            let wy = p.y.floor() as i32;
            let wz = p.z.floor() as i32;
            if self.voxel_at(wx, wy, wz) != VoxelKind::Air {
                return Some((wx, wy, wz));
            }
            t += step;
        }
        None
    }

    /// 射线方向上，紧贴首个固体前的空气格，用于放置。
    pub fn raycast_place_cell(
        &self,
        origin: Vector3,
        dir: Vector3,
        max_dist: f32,
    ) -> Option<(i32, i32, i32)> {
        let dir_len = dir.length();
        if dir_len < 1e-6 {
            return None;
        }
        let dir = dir * (1.0 / dir_len);
        let mut t = 0.08_f32;
        let step = 0.07_f32;
        let mut last_air: Option<(i32, i32, i32)> = None;
        while t < max_dist {
            let p = origin + dir * t;
            let wx = p.x.floor() as i32;
            let wy = p.y.floor() as i32;
            let wz = p.z.floor() as i32;
            let cell = (wx, wy, wz);
            if self.voxel_at(wx, wy, wz) != VoxelKind::Air {
                return last_air;
            }
            last_air = Some(cell);
            t += step;
        }
        None
    }

    /// 当玩家进入新区块时重建可见范围合并网格（程序化地形，水平方向无硬边界）。
    ///
    /// **仅区块中心变化**时走后台线程，避免边跑图边算大图卡逻辑帧。体素编辑后的网格刷新请用
    /// [`Self::flush_voxel_mesh_sync`]，不要依赖本函数的 `mesh_rebuild_queued` 分支（已移除）。
    pub fn update_streaming(&mut self, player_position: &Vector3) {
        let wx = player_position.x.floor() as i32;
        let wy = player_position.y.floor() as i32;
        let (cx, cy) = world_chunk_coords(wx, wy);

        let join_finished = match &self.mesh_build_join {
            Some(h) => h.is_finished(),
            None => false,
        };
        if join_finished {
            let handle = self.mesh_build_join.take().unwrap();
            match handle.join() {
                Ok(new_mesh) => {
                    if self.pending_center_chunk == Some((cx, cy)) {
                        *self.mesh.borrow_mut() = new_mesh;
                        self.streaming_center_chunk = Some((cx, cy));
                    }
                    self.pending_center_chunk = None;
                }
                Err(_) => {
                    self.pending_center_chunk = None;
                }
            }
        }

        let need_new_mesh =
            self.mesh_build_join.is_none() && self.streaming_center_chunk != Some((cx, cy));

        if need_new_mesh {
            self.pending_center_chunk = Some((cx, cy));
            let radius = VIEW_RADIUS_CHUNKS;
            let overrides = self.overrides.clone();
            self.mesh_build_join = Some(std::thread::spawn(move || {
                build_world_mesh(cx, cy, radius, overrides)
            }));
        }
    }

    /// 与 `procedural_voxel` 一致：`surface_height` 为该列**最低空气体素**的 z 索引，即草方块顶面所在世界高度。
    /// 角色碰撞体以 AABB **最小角**为原点且底面为 `position.z`，故脚底应放在该高度略上方，避免与顶面相切被判进固体。
    pub fn suggested_spawn() -> Vector3 {
        let sx = 8i32;
        let sy = 8i32;
        let ground_z = surface_height(sx, sy) as f32;
        let z = ground_z + 0.02;
        Vector3::new(sx as f32, sy as f32, z)
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
                    if self.voxel_at(ix, iy, iz) == VoxelKind::Air {
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
