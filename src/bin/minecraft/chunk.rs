use std::rc::Rc;

use runtime::{core::math::{matrix4::Matrix4x4, vector3::Vector3}, function::{framework::component::component::{Component, ComponentTrait}, render::{render_object::GameObjectDynamicMeshDesc, render_type::MeshVertexDataDefinition}}};

use crate::block::{self, BLOCK_TEXTURE_DIM, Block, BlockType, FACE_DIRECTION_OFFSETS};

pub const CHUNK_DIM: (u32, u32, u32) = (16, 16, 256);
const CHUNK_SIZE: usize = (CHUNK_DIM.0 * CHUNK_DIM.1 * CHUNK_DIM.2) as usize;

fn get_chunk_offset(x: u32, y: u32, z: u32) -> usize {
    (z + CHUNK_DIM.2 * (y + CHUNK_DIM.1 * x)) as usize
}

#[derive(Clone)]
pub struct ChunkData {
    air_block: Rc<Block>,
    pub blocks: [Rc<Block>; CHUNK_SIZE],
}

impl Default for ChunkData {
    fn default() -> Self {
        let air_block = Rc::new(Block {
            ..Default::default()
        });
        Self { 
            air_block: air_block.clone(),
            blocks: {
                std::array::from_fn(|_| air_block.clone())
            },
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    pub m_component: Component,
    pub data: ChunkData,
}

impl ComponentTrait for Chunk  {
    fn get_component(&self) ->  &Component {
        &self.m_component
    }

    fn get_component_mut(&mut self) ->  &mut Component {
        &mut self.m_component
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ComponentTrait> {
        Box::new(self.clone())
    }
}

impl Chunk {

    pub fn new_box() -> Box<Self> {
        let mut res = Box::<Self>::new_uninit();
        unsafe {
            let air_block = Rc::new(Block {
                m_block_type: BlockType::Air,
                ..Default::default()
            });
            std::ptr::write(&mut (*res.as_mut_ptr()).data.air_block, Rc::clone(&air_block));
            for i in 0..CHUNK_SIZE {
                std::ptr::write(&mut (*res.as_mut_ptr()).data.blocks[i], Rc::clone(&air_block));
            }
            res.assume_init()
        }
    }

    pub fn update_mesh_data(&self) -> GameObjectDynamicMeshDesc {
        let mut res = GameObjectDynamicMeshDesc::default();
        res.m_is_dirty = true;
        for i in 0..CHUNK_DIM.0 as i32 {
            for j in 0..CHUNK_DIM.1 as i32 {
                for k in 0..CHUNK_DIM.2 as i32 {
                    let offset = get_chunk_offset(i as u32, j as u32, k as u32);
                    let block = &self.data.blocks[offset];
                    if let BlockType::Air = block.m_block_type {
                        continue;
                    }
                    for direction in 0..6 {
                        let neighbor = (
                            i + FACE_DIRECTION_OFFSETS[direction].0, 
                            j + FACE_DIRECTION_OFFSETS[direction].1, 
                            k + FACE_DIRECTION_OFFSETS[direction].2 
                        );
                        if neighbor.0 < 0 || neighbor.0 >= CHUNK_DIM.0 as i32 ||
                           neighbor.1 < 0 || neighbor.1 >= CHUNK_DIM.1 as i32 ||
                           neighbor.2 < 0 || neighbor.2 >= CHUNK_DIM.2 as i32 {

                            let translate = Vector3::new(i as f32, j as f32, k as f32).to_translate_matrix();
                            let mut face = block::FACES[direction];
                            face.iter_mut().for_each(|vertex|{
                                transform_vertex(vertex, &translate, &(block.get_texture_location)((direction as u32).try_into().unwrap()));
                            });
                            let indices = block::INDICES.iter().map(|index| index + res.m_vertices.len() as u32).collect::<Vec<_>>();
                            res.m_vertices.extend_from_slice(&face);
                            res.m_indices.extend_from_slice(&indices);
                        }
                        else{
                            let neighbor_offset = get_chunk_offset(neighbor.0 as u32, neighbor.1 as u32, neighbor.2 as u32);
                            let neighbor_block = &self.data.blocks[neighbor_offset];
                            if let BlockType::Air = neighbor_block.m_block_type {
                                let translate = Vector3::new(i as f32, j as f32, k as f32).to_translate_matrix();
                                let mut face = block::FACES[direction];
                                face.iter_mut().for_each(|vertex|{
                                    transform_vertex(vertex, &translate, &(block.get_texture_location)((direction as u32).try_into().unwrap()));
                                });
                                let indices = block::INDICES.iter().map(|index| index + res.m_vertices.len() as u32).collect::<Vec<_>>();
                                res.m_indices.extend_from_slice(&indices);
                                res.m_vertices.extend_from_slice(&face);
                            } else {
                                continue; // Neighbor is solid, skip face
                            }
                        }
                    }
                }
            }
        }
        res
    }

    pub fn set_block(&mut self, x: u32, y: u32, z: u32, block: &Rc<Block>) {
        let offset = get_chunk_offset(x, y, z);
        self.data.blocks[offset] = Rc::clone(block);
    }

    pub fn fill(&mut self, x1: u32, y1: u32, z1: u32, x2: u32, y2: u32, z2: u32, block: &Rc<Block>) {
        for x in x1..x2 {
            for y in y1..y2 {
                for z in z1..z2 {
                    self.set_block(x, y, z, block);
                }
            }
        }
    }
}

fn transform_vertex(vertex: &mut MeshVertexDataDefinition, translate: &Matrix4x4, texture_offset: &(u32, u32)) {
    let pos = Vector3::new(vertex.x, vertex.y, vertex.z);
    let pos =  translate * pos.to_homogeneous();
    vertex.x = pos.x;
    vertex.y = pos.y;
    vertex.z = pos.z;

    vertex.u = (vertex.u + texture_offset.0 as f32) / BLOCK_TEXTURE_DIM.0 as f32;
    vertex.v = (vertex.v + texture_offset.1 as f32) / BLOCK_TEXTURE_DIM.1 as f32;
}