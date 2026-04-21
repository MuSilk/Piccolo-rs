//! 体素世界：程序化高度场 + 以玩家为中心的环形区块网格（单动态网格合并），碰撞对任意世界坐标采样，地形在水平方向可无限延伸。
//!
//! 资源使用 `editor/asset/minecraft/block.json`（运行时路径 `asset/minecraft/block.json`）。
//!
//! 区块切换时网格合并 `build_world_mesh` 计算量大；在后台线程构建、主线程每帧取回结果，避免逻辑帧长时间卡住。

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use runtime::{
    core::math::{
        axis_aligned::AxisAlignedBox, matrix4::Matrix4x4, transform::Transform, vector3::Vector3,
    },
    engine::Engine,
    function::{
        framework::{
            component::{
                component::ComponentTrait, mesh::mesh_component::MeshComponent,
                transform_component::TransformComponent,
            },
            object::object_id_allocator::GObjectID,
            resource::component::mesh::MeshComponentRes,
            scene::scene::Scene,
        },
        render::{
            render_object::{GameObjectDynamicMeshDesc, GameObjectMeshDesc, GameObjectPartDesc},
            render_type::MeshVertexDataDefinition,
        },
    },
};
use serde::{Deserialize, Serialize};

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
const VIEW_RADIUS_CHUNKS: i32 = 7;

const ATLAS: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Plank = 5,
    Brick = 6,
    Log = 7,
    #[serde(alias = "Glow")]
    Leaves = 8,
}

fn world_chunk_coords(wx: i32, wy: i32) -> (i32, i32) {
    (wx.div_euclid(CHUNK_SIZE), wy.div_euclid(CHUNK_SIZE))
}

/// 格点伪随机 \([0,1)\)，无三角函数，地形在大尺度上不再呈简单周期。
fn lattice_noise01(ix: i32, iy: i32) -> f32 {
    let mut n = (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA6B);
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

fn lattice_noise01_3d(ix: i32, iy: i32, iz: i32) -> f32 {
    let mut n = (ix as u32).wrapping_mul(0x9E37_79B1)
        ^ (iy as u32).wrapping_mul(0x85EB_CA6B)
        ^ (iz as u32).wrapping_mul(0xC2B2_AE35);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_A68B);
    n ^= n >> 16;
    (n as f32) * (1.0 / u32::MAX as f32)
}

fn value_noise_3d(x: f32, y: f32, z: f32) -> f32 {
    let x0f = x.floor();
    let y0f = y.floor();
    let z0f = z.floor();
    let x0 = x0f as i32;
    let y0 = y0f as i32;
    let z0 = z0f as i32;
    let tx = smoothstep3(x - x0f);
    let ty = smoothstep3(y - y0f);
    let tz = smoothstep3(z - z0f);

    let c000 = lattice_noise01_3d(x0, y0, z0);
    let c100 = lattice_noise01_3d(x0 + 1, y0, z0);
    let c010 = lattice_noise01_3d(x0, y0 + 1, z0);
    let c110 = lattice_noise01_3d(x0 + 1, y0 + 1, z0);
    let c001 = lattice_noise01_3d(x0, y0, z0 + 1);
    let c101 = lattice_noise01_3d(x0 + 1, y0, z0 + 1);
    let c011 = lattice_noise01_3d(x0, y0 + 1, z0 + 1);
    let c111 = lattice_noise01_3d(x0 + 1, y0 + 1, z0 + 1);

    let x00 = c000 + (c100 - c000) * tx;
    let x10 = c010 + (c110 - c010) * tx;
    let x01 = c001 + (c101 - c001) * tx;
    let x11 = c011 + (c111 - c011) * tx;
    let y0v = x00 + (x10 - x00) * ty;
    let y1v = x01 + (x11 - x01) * ty;
    y0v + (y1v - y0v) * tz
}

const CAVE_FREQ_1_XY: f32 = 0.058;
const CAVE_FREQ_1_Z: f32 = 0.078;
const CAVE_FREQ_2_XY: f32 = 0.112;
const CAVE_FREQ_2_Z: f32 = 0.145;
const CAVE_DEEP_THRESHOLD: f32 = 0.66;
const CAVE_SURFACE_MOUTH_THRESHOLD: f32 = 0.80;
const CAVE_SURFACE_LAYER_TOP_OFFSET: i32 = 1;
const CAVE_SURFACE_LAYER_BOTTOM_OFFSET: i32 = 6;

fn is_cave_air(wx: i32, wy: i32, wz: i32, top: i32) -> bool {
    if wz < 4 {
        return false;
    }
    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;

    // 两层 3D 噪声叠加形成连通洞穴（频率略降低，让洞更大、更容易观察到）。
    let n1 = value_noise_3d(x * CAVE_FREQ_1_XY, y * CAVE_FREQ_1_XY, z * CAVE_FREQ_1_Z);
    let n2 = value_noise_3d(
        x * CAVE_FREQ_2_XY + 37.0,
        y * CAVE_FREQ_2_XY - 11.0,
        z * CAVE_FREQ_2_Z + 5.0,
    );
    let cave = n1 * 0.72 + n2 * 0.28;

    // 地表以下较深处：正常洞穴密度。
    if wz <= top - (CAVE_SURFACE_LAYER_BOTTOM_OFFSET + 1) {
        return cave > CAVE_DEEP_THRESHOLD;
    }

    // 近地表薄层（top-6..top-1）：用独立 2D 掩码决定“天窗/井口”，确保可见洞口但不过量。
    if wz <= top - CAVE_SURFACE_LAYER_TOP_OFFSET {
        let mouth_mask = value_noise_2d(x * 0.05 + 31.0, y * 0.05 - 17.0);
        let mouth_depth_noise = value_noise_2d(x * 0.09 - 7.0, y * 0.09 + 23.0);
        let mouth_depth = 3 + (mouth_depth_noise * 6.0) as i32; // 3..8
        let in_mouth_depth = wz >= top - mouth_depth && wz <= top - CAVE_SURFACE_LAYER_TOP_OFFSET;
        return mouth_mask > CAVE_SURFACE_MOUTH_THRESHOLD && in_mouth_depth;
    }

    false
}

const TREE_GRID: i32 = 7;
const TREE_OFFSET: i32 = 3;

fn trunk_height_at(anchor_x: i32, anchor_y: i32) -> i32 {
    // 4..6
    4 + (lattice_noise01(anchor_x * 31 + 17, anchor_y * 29 + 41) * 3.0) as i32
}

fn has_tree_at(anchor_x: i32, anchor_y: i32) -> bool {
    let top = surface_height(anchor_x, anchor_y);
    if top < 16 {
        return false;
    }
    let n = lattice_noise01(anchor_x * 13 + 7, anchor_y * 11 + 19);
    n > 0.62
}

fn tree_voxel(wx: i32, wy: i32, wz: i32) -> VoxelKind {
    // 检查可能影响当前体素的邻近树锚点（树叶半径 2，格子间距 7）。
    let gx0 = (wx - 2 - TREE_OFFSET).div_euclid(TREE_GRID);
    let gx1 = (wx + 2 - TREE_OFFSET).div_euclid(TREE_GRID);
    let gy0 = (wy - 2 - TREE_OFFSET).div_euclid(TREE_GRID);
    let gy1 = (wy + 2 - TREE_OFFSET).div_euclid(TREE_GRID);
    for gx in gx0..=gx1 {
        for gy in gy0..=gy1 {
            let ax = gx * TREE_GRID + TREE_OFFSET;
            let ay = gy * TREE_GRID + TREE_OFFSET;
            if !has_tree_at(ax, ay) {
                continue;
            }

            let top = surface_height(ax, ay);
            let trunk_h = trunk_height_at(ax, ay);
            let trunk_base = top;
            let trunk_top = trunk_base + trunk_h - 1;

            if wx == ax && wy == ay && wz >= trunk_base && wz <= trunk_top {
                return VoxelKind::Log;
            }

            // 树冠：位于树干顶部附近，近似椭球体，且不覆盖树干本体。
            let crown_cz = trunk_top + 1;
            let dx = (wx - ax).abs();
            let dy = (wy - ay).abs();
            let dz = (wz - crown_cz).abs();
            let in_crown = dx <= 2
                && dy <= 2
                && dz <= 2
                && (dx + dy + dz <= 4 || (dx <= 1 && dy <= 1 && dz <= 2));
            if in_crown && !(wx == ax && wy == ay && wz <= trunk_top + 1) {
                return VoxelKind::Leaves;
            }
        }
    }
    VoxelKind::Air
}

fn procedural_voxel(wx: i32, wy: i32, wz: i32) -> VoxelKind {
    if wz < 0 || wz >= WZ {
        return VoxelKind::Air;
    }
    let top = surface_height(wx, wy);
    if wz >= top {
        return tree_voxel(wx, wy, wz);
    }
    if is_cave_air(wx, wy, wz, top) {
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
            5 => (2, 0), // -Z 底面
            _ => (0, 0), // 四个侧面
        },
        VoxelKind::Dirt => (2, 0),
        VoxelKind::Stone => (3, 0),
        VoxelKind::Sand => (4, 0),
        VoxelKind::Plank => (5, 0),
        VoxelKind::Brick => (6, 0),
        // 原木：顶/底与侧面使用不同格子（暂用现有图集占位）。
        VoxelKind::Log => match face {
            4 | 5 => (8, 0),
            _ => (7, 0),
        },
        // 树叶先按不透明处理；材质仍走当前 block.material.json。
        VoxelKind::Leaves => (9, 0),
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

/// 在体素格 `(wx,wy,wz)` 上为外露面生成四顶点；`origin` 可是世界坐标（合并网格）或 chunk 局部坐标。
fn emit_faces_for_cell(
    origin: Vector3,
    wx: i32,
    wy: i32,
    wz: i32,
    kind: VoxelKind,
    get: impl Fn(i32, i32, i32) -> VoxelKind,
    verts: &mut Vec<MeshVertexDataDefinition>,
    idx: &mut Vec<u32>,
) {
    let o = origin;
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

fn build_chunk_mesh(
    chunk_cx: i32,
    chunk_cy: i32,
    overrides: &HashMap<(i32, i32, i32), VoxelKind>,
) -> GameObjectDynamicMeshDesc {
    let mut verts = Vec::new();
    let mut indices = Vec::new();
    let get = |x: i32, y: i32, z: i32| {
        if let Some(&k) = overrides.get(&(x, y, z)) {
            k
        } else {
            procedural_voxel(x, y, z)
        }
    };
    let x0 = chunk_cx * CHUNK_SIZE;
    let y0 = chunk_cy * CHUNK_SIZE;
    for lz in 0..WZ {
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let wx = x0 + lx;
                let wy = y0 + ly;
                let k = get(wx, wy, lz);
                if k == VoxelKind::Air {
                    continue;
                }
                emit_faces_for_cell(
                    Vector3::new(lx as f32, ly as f32, lz as f32),
                    wx,
                    wy,
                    lz,
                    k,
                    &get,
                    &mut verts,
                    &mut indices,
                );
            }
        }
    }
    let mut mesh = GameObjectDynamicMeshDesc::default();
    mesh.m_is_dirty = true;
    mesh.m_mesh_file = format!("minecraft_chunk_{}_{}.mesh", chunk_cx, chunk_cy);
    mesh.m_vertices = verts;
    mesh.m_indices = indices;
    mesh
}

struct ChunkRenderEntry {
    object_id: GObjectID,
    mesh: Rc<RefCell<GameObjectDynamicMeshDesc>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoxelOverrideRecord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub kind: VoxelKind,
}

pub struct VoxelWorld {
    /// 相对程序化地形的体素覆盖（破坏/放置）；与程序化一致时可 `remove` 节省内存。
    overrides: HashMap<(i32, i32, i32), VoxelKind>,
    mesh_res: MeshComponentRes,
    loaded_chunks: HashMap<(i32, i32), ChunkRenderEntry>,
    dirty_chunks: HashSet<(i32, i32)>,
}

impl VoxelWorld {
    fn create_chunk_object(&mut self, engine: &Engine, scene: &mut Scene, cx: i32, cy: i32) {
        let mesh = Rc::new(RefCell::new(build_chunk_mesh(cx, cy, &self.overrides)));
        let object_id = scene.spawn();
        let mut mesh_component = Box::new(MeshComponent::default());
        mesh_component.post_load_resource(object_id, engine.asset_manager(), &self.mesh_res);
        mesh_component
            .m_raw_meshes
            .resize(1, GameObjectPartDesc::default());
        mesh_component.m_raw_meshes[0].m_mesh_desc =
            GameObjectMeshDesc::DynamicMesh(Rc::clone(&mesh));

        let mut transform = Box::new(TransformComponent::default());
        transform.post_load_resource(Transform::new(
            Vector3::new((cx * CHUNK_SIZE) as f32, (cy * CHUNK_SIZE) as f32, 0.0),
            runtime::core::math::quaternion::Quaternion::identity(),
            Vector3::ONES,
        ));

        scene.create_object(
            object_id,
            vec![
                RefCell::new(mesh_component) as RefCell<Box<dyn ComponentTrait>>,
                RefCell::new(transform),
            ],
        );
        self.loaded_chunks
            .insert((cx, cy), ChunkRenderEntry { object_id, mesh });
    }

    fn rebuild_loaded_chunk(&mut self, cx: i32, cy: i32) {
        if let Some(entry) = self.loaded_chunks.get(&(cx, cy)) {
            *entry.mesh.borrow_mut() = build_chunk_mesh(cx, cy, &self.overrides);
        }
    }

    pub fn new_box(engine: &Engine, scene: &mut Scene) -> Box<Self> {
        let mesh_res = engine
            .asset_manager()
            .load_asset::<BlockMeshJson>("asset/minecraft/block.json")
            .expect("asset/minecraft/block.json")
            .m_mesh_res;
        let mut world = Self {
            overrides: HashMap::new(),
            mesh_res,
            loaded_chunks: HashMap::new(),
            dirty_chunks: HashSet::new(),
        };
        let spawn_xy = (8i32, 8i32);
        let (icx, icy) = world_chunk_coords(spawn_xy.0, spawn_xy.1);
        for cy in (icy - VIEW_RADIUS_CHUNKS)..=(icy + VIEW_RADIUS_CHUNKS) {
            for cx in (icx - VIEW_RADIUS_CHUNKS)..=(icx + VIEW_RADIUS_CHUNKS) {
                world.create_chunk_object(engine, scene, cx, cy);
            }
        }
        Box::new(world)
    }

    #[inline]
    pub fn voxel_at(&self, wx: i32, wy: i32, wz: i32) -> VoxelKind {
        if let Some(&k) = self.overrides.get(&(wx, wy, wz)) {
            k
        } else {
            procedural_voxel(wx, wy, wz)
        }
    }

    pub fn snapshot_overrides(&self) -> Vec<VoxelOverrideRecord> {
        self.overrides
            .iter()
            .map(|(&(x, y, z), &kind)| VoxelOverrideRecord { x, y, z, kind })
            .collect()
    }

    pub fn replace_overrides(&mut self, overrides: Vec<VoxelOverrideRecord>) {
        self.overrides.clear();
        for rec in overrides {
            if rec.kind == procedural_voxel(rec.x, rec.y, rec.z) {
                continue;
            }
            self.overrides.insert((rec.x, rec.y, rec.z), rec.kind);
        }
        self.dirty_chunks.extend(self.loaded_chunks.keys().copied());
    }

    pub fn set_voxel(&mut self, wx: i32, wy: i32, wz: i32, kind: VoxelKind) {
        if kind == procedural_voxel(wx, wy, wz) {
            self.overrides.remove(&(wx, wy, wz));
        } else {
            self.overrides.insert((wx, wy, wz), kind);
        }

        let (cx, cy) = world_chunk_coords(wx, wy);
        self.dirty_chunks.insert((cx, cy));
        let lx = wx.rem_euclid(CHUNK_SIZE);
        let ly = wy.rem_euclid(CHUNK_SIZE);
        if lx == 0 {
            self.dirty_chunks.insert((cx - 1, cy));
        } else if lx == CHUNK_SIZE - 1 {
            self.dirty_chunks.insert((cx + 1, cy));
        }
        if ly == 0 {
            self.dirty_chunks.insert((cx, cy - 1));
        } else if ly == CHUNK_SIZE - 1 {
            self.dirty_chunks.insert((cx, cy + 1));
        }
    }

    /// 将 `set_voxel` 累积的修改立刻反映到合并网格（主线程同步构建）。
    ///
    /// 破坏/放置若仍走后台线程，要等整幅网格算完才有画面，体感延迟大；编辑后应优先调用本函数。
    /// 若仍有未完成的异步区块构建，会先 `join` 丢弃其结果，再以当前 `overrides` 与玩家所在区块中心重建。
    pub fn flush_voxel_mesh_sync(&mut self, _player_position: &Vector3) {
        let dirty: Vec<(i32, i32)> = self.dirty_chunks.drain().collect();
        for (cx, cy) in dirty {
            self.rebuild_loaded_chunk(cx, cy);
        }
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
    pub fn update_streaming(
        &mut self,
        engine: &Engine,
        scene: &mut Scene,
        player_position: &Vector3,
    ) {
        let wx = player_position.x.floor() as i32;
        let wy = player_position.y.floor() as i32;
        let (cx, cy) = world_chunk_coords(wx, wy);
        let mut target = HashSet::new();
        for y in (cy - VIEW_RADIUS_CHUNKS)..=(cy + VIEW_RADIUS_CHUNKS) {
            for x in (cx - VIEW_RADIUS_CHUNKS)..=(cx + VIEW_RADIUS_CHUNKS) {
                target.insert((x, y));
            }
        }

        let loaded_keys: Vec<(i32, i32)> = self.loaded_chunks.keys().copied().collect();
        for key in loaded_keys {
            if !target.contains(&key) {
                if let Some(entry) = self.loaded_chunks.remove(&key) {
                    scene.delete_object_by_id(engine, entry.object_id);
                }
                self.dirty_chunks.remove(&key);
            }
        }

        for key in target {
            if !self.loaded_chunks.contains_key(&key) {
                self.create_chunk_object(engine, scene, key.0, key.1);
            }
        }

        self.flush_voxel_mesh_sync(player_position);
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
