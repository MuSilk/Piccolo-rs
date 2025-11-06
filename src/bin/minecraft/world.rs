use std::{cell::RefCell, rc::Rc};

use runtime::{core::{algorithm::noise, math::{transform::Transform, vector3::Vector3}}, function::{framework::{component::{component::{Component, ComponentTrait}, mesh::mesh_component::MeshComponent, transform::transform_component::TransformComponent}, scene::scene::Scene}, global::global_context::RuntimeGlobalContext, render::render_object::{GameObjectMeshDesc, GameObjectPartDesc}}};

use crate::{block::{BLOCK_DIRT, BLOCK_GRASS, BLOCK_STONE}, block_res::BlockRes, chunk::Chunk};


#[derive(Clone)]
pub struct World {
    pub m_component: Component,
    pub loaded_chunks: [[Box<Chunk>;12];12],
    seed: f32,
}

impl ComponentTrait for World  {
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

impl World {
    pub fn new_box(level: &mut Scene) -> Box<Self> {

        let mut world = Self {
            m_component: Component::default(),
            seed: rand::random(),
            loaded_chunks: std::array::from_fn(|_i|{
                std::array::from_fn(|_j| Chunk::new_box())
            }),
        };
        
        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        let block: BlockRes = assert_manager.load_asset("asset/minecraft/block.json").unwrap();
        let dirt_block = Rc::new(BLOCK_DIRT);
        let grass_block = Rc::new(BLOCK_GRASS);
        let stone_block = Rc::new(BLOCK_STONE);
        println!("world seed: {}", world.seed);

        let bs = [
            noise::basic(1), noise::basic(2),
            noise::basic(3), noise::basic(4),
        ];

        let os = [
            noise::octave(5, 0), noise::octave(5, 1),
            noise::octave(5, 2), noise::octave(5, 3),
            noise::octave(5, 4), noise::octave(5, 5),
        ];

        let cs = [
            noise::combined(bs[0].clone_box(), bs[1].clone_box()),
            noise::combined(bs[2].clone_box(), bs[3].clone_box()),
            noise::combined(os[3].clone_box(), os[4].clone_box()),
            noise::combined(os[1].clone_box(), os[2].clone_box()),
            noise::combined(os[1].clone_box(), os[3].clone_box()),
        ];

        let n_h = noise::exp_scale(os[0].clone_box(), 1.3, 1.0 / 128.0);
        let n_m = noise::exp_scale(cs[0].clone_box(), 1.0, 1.0 / 512.0);
        let n_t = noise::exp_scale(cs[1].clone_box(), 1.0, 1.0 / 512.0);
        let n_r = noise::exp_scale(cs[2].clone_box(), 1.0, 1.0 / 16.0);
        let n_n = noise::exp_scale(cs[3].clone_box(), 3.0, 1.0 / 512.0);
        let n_p = noise::exp_scale(cs[4].clone_box(), 3.0, 1.0 / 512.0);

        for i in 0..12 {
            for j in 0..12 {
                let object_id = level.spawn();
                let chunk = &mut world.loaded_chunks[i][j];

                for chunk_i in 0..16 {
                    for chunk_j in 0..16 { 
                        let wx = (i * 16 + chunk_i) as f32;
                        let wy = (j * 16 + chunk_j) as f32;

                        let mut h = n_h.compute(wx, wy, world.seed);
                        let r = n_r.compute(wx, wy, world.seed);
                        let mut n = n_n.compute(wx, wy, world.seed);
                        let p = n_p.compute(wx, wy, world.seed);

                        let exp = 1.0;
                        let scale = 1.0;
                        let roughness = 1.0;

                        n += p.signum() * p.abs().powf((1.0 - n) * 3.0);

                        h = h.signum() * h.abs().powf(exp);

                        let height = (h * 32.0 + n * 256.0) * scale + r * roughness * 2.0;
                        let height = height.min(255.0).max(1.0) as i32;

                        for h in 0..=height {
                            if h == height {
                                chunk.set_block(chunk_i as u32, chunk_j as u32, h as u32, &grass_block);
                            }
                            else if h >= height - 3 {
                                chunk.set_block(chunk_i as u32, chunk_j as u32, h as u32, &dirt_block);
                            }
                            else{
                                chunk.set_block(chunk_i as u32, chunk_j as u32, h as u32, &stone_block);
                            }
                        }
                    }
                }
                
                let mut mesh_component = Box::new(MeshComponent::default());
                mesh_component.post_load_resource(object_id, &block.m_mesh_res);
                let mut chunk_data = chunk.update_mesh_data();
                chunk_data.m_mesh_file = format!("chunk_{}_{}.mesh", i, j);
                mesh_component.m_raw_meshes.resize(1, GameObjectPartDesc::default());
                mesh_component.m_raw_meshes[0].m_mesh_desc = GameObjectMeshDesc::DynamicMesh(Rc::new(RefCell::new(chunk_data)));
                let mut transform_component = Box::new(TransformComponent::default());
                transform_component.post_load_resource(object_id, Transform::default());
                transform_component.set_position(Vector3::new(i as f32 * 16.0, j as f32 * 16.0, 0.0));
                let components = vec![
                    RefCell::new(mesh_component) as RefCell<Box<dyn ComponentTrait>>,
                    RefCell::new(transform_component),
                ];
                level.create_object(object_id, components);
            }
        }
        Box::new(world)
    }
}