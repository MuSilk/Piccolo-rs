use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{core::math::vector3::Vector3, function::{framework::{component::{component::{Component, ComponentTrait}, mesh::mesh_component::MeshComponent, transform::transform_component::TransformComponent}, level::level::Level, minecraft::{block::Block, chunk::Chunk}, object::{object::GObject, object_id_allocator}}, global::global_context::RuntimeGlobalContext}};

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    #[serde(skip)]
    pub m_component: Component,
    pub loaded_chunks: [[Box<Chunk>;5];5]
}

#[typetag::serde]
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
    pub fn new_box(level: &Rc<RefCell<Level>>) -> Box<Self> {

        let mut world = Self {
            m_component: Component::default(),
            loaded_chunks: std::array::from_fn(|_i|{
                std::array::from_fn(|j| Chunk::new_box())
            }),
        };
        
        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        let mut dirt_block: Block = assert_manager.load_asset("asset/minecraft/dirt.block.json").unwrap();
        dirt_block.post_load_resource(&level, 0);
        let dirt_block = Rc::new(dirt_block);

        for i in 0..5 {
            for j in 0..5 {
                let object_id = object_id_allocator::alloc();
                let gobject = GObject::new(object_id);
                level.borrow_mut().m_entities.insert(object_id, gobject);
                let chunk = &mut world.loaded_chunks[i][j];
                chunk.fill(0, 0, 0, 16, 16, 64, &dirt_block);
                let mut mesh_component = Box::new(MeshComponent::default());
                chunk.update_mesh_component(&mut mesh_component);
                mesh_component.post_load_resource(&level, object_id);
                let mut transform_component = Box::new(TransformComponent::default());
                transform_component.post_load_resource(level, object_id);
                transform_component.set_position(Vector3::new(i as f32 * 16.0, j as f32 * 16.0, 0.0));
                let components = vec![
                    RefCell::new(mesh_component) as RefCell<Box<dyn ComponentTrait>>,
                    RefCell::new(transform_component),
                ];
                level.borrow_mut().create_object(object_id, components);
            }
        }
        Box::new(world)
    }
}