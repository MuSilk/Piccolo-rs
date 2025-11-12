use runtime::function::render::render_type::MeshVertexDataDefinition;

pub enum FaceDirection {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl TryFrom<u32> for FaceDirection {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == FaceDirection::Top as u32 => Ok(FaceDirection::Top),
            x if x == FaceDirection::Bottom as u32 => Ok(FaceDirection::Bottom),
            x if x == FaceDirection::Left as u32 => Ok(FaceDirection::Left),
            x if x == FaceDirection::Right as u32 => Ok(FaceDirection::Right),
            x if x == FaceDirection::Front as u32 => Ok(FaceDirection::Front),
            x if x == FaceDirection::Back as u32 => Ok(FaceDirection::Back),
            _ => Err(()),
        }
    }
}

pub const FACE_DIRECTION_OFFSETS: [(i32, i32, i32); 6] = [
    ( 0, 0, 1),  // Top
    ( 0, 0,-1),  // Bottom
    (-1, 0, 0),  // Left
    ( 1, 0, 0),  // Right
    ( 0, 1, 0),  // Front
    ( 0,-1, 0),  // Back
];

const TOP_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
        x: 0.0,  y: 0.0,  z: 1.0,
        nx: 0.0, ny: 0.0, nz: 1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  0.0, v: 0.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 1.0,
        nx: 0.0, ny: 0.0, nz: 1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  0.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 1.0,
        nx: 0.0, ny: 0.0, nz: 1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 0.0, z: 1.0,
        nx: 0.0, ny: 0.0, nz: 1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 0.0
    }
];

const BOTTOM_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
        x: 0.0, y: 0.0, z: 0.0,
        nx: 0.0, ny: 0.0, nz: -1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  0.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 0.0, z: 0.0,
        nx: 0.0, ny: 0.0, nz: -1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 0.0,
        nx: 0.0, ny: 0.0, nz: -1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 0.0,
        nx: 0.0, ny: 0.0, nz: -1.0,
        tx: 1.0, ty: 0.0, tz: 0.0,
        u:  0.0,  v: 1.0
    }
];

const LEFT_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
        x: 0.0, y: 0.0, z: 1.0,
        nx: -1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: -1.0, tz: 1.0,
        u:  1.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 0.0, z: 0.0,
        nx: -1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: -1.0, tz: 1.0,
        u:  1.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 0.0,
        nx: -1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: -1.0, tz: 1.0,
        u:  0.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 1.0,
        nx: -1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: -1.0, tz: 1.0,
        u:  0.0,  v: 0.0
    }
];

const RIGHT_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
        x: 1.0, y: 0.0, z: 0.0,
        nx: 1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: 1.0, tz: 0.0,
        u:  0.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 0.0, z: 1.0,
        nx: 1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: 1.0, tz: 0.0,
        u:  0.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 1.0,
        nx: 1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: 1.0, tz: 0.0,
        u:  1.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 0.0,
        nx: 1.0, ny: 0.0, nz: 0.0,
        tx: 0.0, ty: 1.0, tz: 0.0,
        u:  1.0,  v: 1.0
    }
];

const FRONT_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 0.0,
        nx: 0.0, ny: 1.0, nz: 0.0,
        tx: -1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 0.0,
        nx: 0.0, ny: 1.0, nz: 0.0,
        tx: -1.0, ty: 0.0, tz: 0.0,
        u:  0.0,  v: 1.0
    },
    MeshVertexDataDefinition{
        x: 1.0, y: 1.0, z: 1.0,
        nx: 0.0, ny: 1.0, nz: 0.0,
        tx: -1.0, ty: 0.0, tz: 0.0,
        u:  0.0,  v: 0.0
    },
    MeshVertexDataDefinition{
        x: 0.0, y: 1.0, z: 1.0,
        nx: 0.0, ny: 1.0, nz: 0.0,
        tx: -1.0, ty: 0.0, tz: 0.0,
        u:  1.0,  v: 0.0
    }
];

const BACK_FACE: [MeshVertexDataDefinition; 4] = [
    MeshVertexDataDefinition{
            x: 0.0, y: 0.0, z: 1.0,
            nx: 0.0, ny: -1.0, nz: 0.0,
            tx: 1.0, ty: 0.0, tz: 0.0,
            u:  0.0,  v: 0.0
        },
        MeshVertexDataDefinition{
            x: 1.0, y: 0.0, z: 1.0,
            nx: 0.0, ny: -1.0, nz: 0.0,
            tx: 1.0, ty: 0.0, tz: 0.0,
            u:  1.0,  v: 0.0
        },
        MeshVertexDataDefinition{
            x: 1.0, y: 0.0, z: 0.0,
            nx: 0.0, ny: -1.0, nz: 0.0,
            tx: 1.0, ty: 0.0, tz: 0.0,
            u:  1.0,  v: 1.0
        },
        MeshVertexDataDefinition{
            x: 0.0, y: 0.0, z: 0.0,
            nx: 0.0, ny: -1.0, nz: 0.0,
            tx: 1.0, ty: 0.0, tz: 0.0,
            u:  0.0,  v: 1.0
        }
];

pub const INDICES:[u32; 6] = [0,1,2,2,3,0];
pub const FACES:[[MeshVertexDataDefinition; 4];6] = [
    TOP_FACE, BOTTOM_FACE, LEFT_FACE, RIGHT_FACE, FRONT_FACE, BACK_FACE
];

#[derive(Clone, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Dirt,
    Grass,
    Stone,
}

#[derive(Clone)]
pub struct Block {
    pub m_block_type: BlockType,
    pub get_texture_location: fn(FaceDirection) -> (u32, u32)
}

pub const BLOCK_TEXTURE_DIM: (u32, u32) = (16, 16);

pub const BLOCK_AIR : Block = Block {
    m_block_type: BlockType::Air,
    get_texture_location: |_d: FaceDirection| (0,0)
};

pub const BLOCK_DIRT : Block = Block {
    m_block_type: BlockType::Dirt,
    get_texture_location: |_d: FaceDirection| (2,0)
};

pub const BLOCK_GRASS: Block = Block {
    m_block_type: BlockType::Grass,
    get_texture_location: |d: FaceDirection| {
        match d {
            FaceDirection::Top | FaceDirection::Bottom => {
                (1,0)
            }
            _ => {
                (0,0)
            }
        }
    }
};

pub const BLOCK_STONE: Block = Block {
    m_block_type: BlockType::Stone,
    get_texture_location: |_d: FaceDirection| (3,0)
};