use std::{any::{TypeId}, cell::{RefCell}, collections::{HashMap, HashSet}, hash::{Hash, Hasher}, rc::Rc};

use anyhow::Result;
use itertools::Itertools;
use log::info;

use crate::{core::math::transform, function::{framework::{component::{component::ComponentTrait, mesh::mesh_component::{self, MeshComponent}, transform::transform_component::TransformComponent}, minecraft::{block::Block, chunk::{Chunk, CHUNK_DIM}}, object::{object::GObject, object_id_allocator::{self, GObjectID}}}, global::global_context::RuntimeGlobalContext, render::render_object::GameObjectDesc}, resource::res_type::common::{level::LevelRes, object::ObjectInstanceRes}};

type ComponentColumn = Vec<RefCell<Box<dyn ComponentTrait>>>;

#[derive(Default)]
struct Archetype {
    m_columns: HashMap<TypeId, ComponentColumn>,
    m_entities: Vec<GObjectID>,
}


impl Archetype {

    fn has_component<T: 'static + ComponentTrait>(&self) -> bool {
        self.m_columns.contains_key(&TypeId::of::<T>())
    }

    fn add_component_type<T: 'static + ComponentTrait>(&mut self) {
        self.m_columns.insert(TypeId::of::<T>(), Vec::new());
    }

    fn add_component_type_by_id(&mut self, type_id: TypeId) {
        self.m_columns.insert(type_id, Vec::new());
    }
    
    fn add_entity(&mut self, object_id: GObjectID, components: Vec<RefCell<Box<dyn ComponentTrait>>>) -> usize {
        assert_eq!(components.iter().map(|c| c.borrow().as_any().type_id()).collect::<HashSet<_>>(),
            self.m_columns.keys().copied().sorted().collect::<HashSet<_>>(),
            "Components do not match archetype!"
        );
        components.into_iter().for_each(|component|{
            let type_id = component.borrow().as_any().type_id(); 
            self.m_columns.get_mut(&type_id).unwrap().push(component);
        });
        self.m_entities.push(object_id);
        self.m_entities.len() - 1
    }

    fn get_entity(&self, index: GObjectID) -> impl Iterator<Item = &RefCell<Box<dyn ComponentTrait>>> {
        self.m_columns.iter()
            .map(move |(_type_id, column)| {
                column.get(index as usize).unwrap()
            })
    }

    fn get_column<T: 'static + ComponentTrait>(&self) -> Option<&ComponentColumn> {
        self.m_columns.get(&TypeId::of::<T>())
    }

}

#[derive(Default)]
pub struct Level {
    m_is_loaded: bool,
    m_level_res_url: String,
    m_archetypes: HashMap<usize, Archetype>,
    m_entity_location: HashMap<GObjectID, (usize, usize)>,
    m_entities: HashMap<GObjectID, Rc<RefCell<GObject>>>,
}

impl Level {
    pub fn new() -> Rc<RefCell<Self>> {
        let level = Self::default();
        let level = Rc::new(RefCell::new(level));
        level
    }
    
    pub fn tick(&mut self, delta_time: f32) {
        if !self.m_is_loaded {
            return;
        }
        self.tick_mesh_components(delta_time);
    }

    fn query<T: 'static + ComponentTrait>(&'_ mut self) -> impl Iterator<Item = std::cell::Ref<'_, T>> {
        self.m_archetypes
            .iter()
            .filter(|(_type_id, archetype)| archetype.has_component::<T>())
            .flat_map(|(_type_id, archetype)| {
                let column = archetype.get_column::<T>().unwrap();
                column.iter().map(|any_box| {
                    std::cell::Ref::map(any_box.borrow(), |b| b.as_any().downcast_ref::<T>().unwrap())
                })
            })
    }

    fn query_mut<T: 'static + ComponentTrait>(&'_ mut self) -> impl Iterator<Item = std::cell::RefMut<'_, T>> {
        self.m_archetypes
            .iter_mut()
            .filter(|(_type_id, archetype)| archetype.has_component::<T>())
            .flat_map(|(_type_id, archetype)| {
                let column = archetype.get_column::<T>().unwrap();
                column.iter().map(|any_box| {
                    std::cell::RefMut::map(any_box.borrow_mut(), |b| b.as_any_mut().downcast_mut::<T>().unwrap())
                })
            })
    }

    fn query_pair<T: 'static + ComponentTrait, U: 'static + ComponentTrait>(&'_ mut self) 
        -> impl Iterator<Item = (std::cell::Ref<'_, T>, std::cell::Ref<'_, U>)> 
    {
        self.m_archetypes
            .iter()
            .filter(|(_type_id, archetype)| archetype.has_component::<T>() && archetype.has_component::<U>())
            .flat_map(|(_type_id, archetype)| {
                let column_t = archetype.get_column::<T>().unwrap();
                let column_u = archetype.get_column::<U>().unwrap();
                column_t.iter().zip(column_u.iter()).map(|(any_box_t, any_box_u)| {
                    (
                        std::cell::Ref::map(any_box_t.borrow(), |b| b.as_any().downcast_ref::<T>().unwrap()),
                        std::cell::Ref::map(any_box_u.borrow(), |b| b.as_any().downcast_ref::<U>().unwrap()),
                    )
                })
            })
    }

    fn query_pair_mut<T: 'static + ComponentTrait, U: 'static + ComponentTrait>(&'_ mut self) 
        -> impl Iterator<Item = (std::cell::RefMut<'_, T>, std::cell::RefMut<'_, U>)> 
    {
        self.m_archetypes
            .iter_mut()
            .filter(|(_type_id, archetype)| archetype.has_component::<T>() && archetype.has_component::<U>())
            .flat_map(|(_type_id, archetype)| {
                let column_t = archetype.get_column::<T>().unwrap();
                let column_u = archetype.get_column::<U>().unwrap();
                column_t.iter().zip(column_u.iter()).map(|(any_box_t, any_box_u)| {
                    (
                        std::cell::RefMut::map(any_box_t.borrow_mut(), |b| b.as_any_mut().downcast_mut::<T>().unwrap()),
                        std::cell::RefMut::map(any_box_u.borrow_mut(), |b| b.as_any_mut().downcast_mut::<U>().unwrap()),
                    )
                })
            })
    }
    
    pub fn create_object(&mut self, object_id: GObjectID, components: Vec<RefCell<Box<dyn ComponentTrait>>>) {
        let archetype_type_id: usize = {
            let mut ids: Vec<_> = components.iter().map(|c| c.borrow().as_any().type_id()).collect();
            ids.sort();
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            ids.hash(&mut hasher);
            hasher.finish() as usize
        };
        if self.m_archetypes.get(&archetype_type_id).is_none() {
            let mut archetype = Archetype::default();
            for component in &components {
                archetype.add_component_type_by_id(component.borrow().as_any().type_id());
            }
            self.m_archetypes.insert(archetype_type_id, archetype);
        }
        let entity_index = self.m_archetypes.get_mut(&archetype_type_id).unwrap().add_entity(object_id, components);
        self.m_entity_location.insert(object_id, (archetype_type_id, entity_index));
    }

    fn tick_mesh_components(&mut self, _delta_time: f32) {
        self.query_pair_mut::<MeshComponent, TransformComponent>()
            .for_each(|(mut mesh, mut transform)| 
        {
            // if transform_component.is_dirty() {
                let mut dirty_mesh_parts = vec![];
                for mesh_part in &mut mesh.m_raw_meshes {
                    let object_transform_matrix = mesh_part.m_transform_desc.m_transform_matrix;

                    mesh_part.m_transform_desc.m_transform_matrix = transform.get_matrix() * object_transform_matrix;
                    dirty_mesh_parts.push(mesh_part.clone());

                    mesh_part.m_transform_desc.m_transform_matrix = object_transform_matrix;
                }

                let render_system = RuntimeGlobalContext::get_render_system().borrow();
                let render_swap_context = render_system.get_swap_context();
                let logic_swap_data = render_swap_context.get_logic_swap_data();
                transform.set_dirty_flag(false);
                logic_swap_data.borrow_mut().add_dirty_game_object(&GameObjectDesc::new(mesh.m_component.m_parent_object, dirty_mesh_parts));
            // }
        });
        
    }

}

pub trait LevelExt {
    fn spawn(&mut self) -> GObjectID;
    fn load(&mut self, level_res_url: &str) -> Result<()>;
    fn save(&self) -> Result<()>;
}

impl LevelExt for Rc<RefCell<Level>> {
    fn spawn(&mut self) -> GObjectID {
        let object_id = object_id_allocator::alloc();
        let gobject = GObject::new(object_id);
        self.borrow_mut().m_entities.insert(object_id, gobject);
        object_id
    }

    fn load(&mut self, level_res_url: &str) -> Result<()> {
        info!("Loading level: {}", level_res_url);
        let level_res = {
            let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
            assert_manager.load_asset::<LevelRes>(level_res_url)?
        };
        level_res.m_objects.iter().for_each(|obj| {
            let object_id = object_id_allocator::alloc();
            let gobject = GObject::new(object_id);
            gobject.borrow_mut().set_name(&obj.m_name);
            gobject.borrow_mut().set_definition_url(&obj.m_definition);
            self.borrow_mut().m_entities.insert(object_id, gobject);

            let components = obj.m_instanced_components.iter().map(|component| {
                let component = RefCell::new(component.clone_box());
                component.borrow_mut().post_load_resource(&self, object_id);
                component
            }).collect::<Vec<_>>();
            self.borrow_mut().create_object(object_id, components);
        });

        let object_id = object_id_allocator::alloc();
        let gobject = GObject::new(object_id);

        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        let mut dirt_block: Block = assert_manager.load_asset("asset/minecraft/dirt.block.json").unwrap();
        dirt_block.post_load_resource(&self, object_id);
        let dirt_block = Rc::new(dirt_block);
        
        self.borrow_mut().m_entities.insert(object_id, gobject);
        let mut chunk = Chunk::new_box();

        // chunk.fill(0, 0, 0, CHUNK_DIM.0, CHUNK_DIM.1, CHUNK_DIM.2, &dirt_block);
        chunk.fill(0, 0, 0, 16, 16, 16, &dirt_block);

        let mut mesh_component = Box::new(MeshComponent::default());
        chunk.update_mesh_component(&mut mesh_component);
        mesh_component.post_load_resource(&self, object_id);
        println!("{}", mesh_component.m_raw_meshes.len());
        let transform_component = Box::new(TransformComponent::default());
        let components = vec![
            RefCell::new(chunk as Box<dyn ComponentTrait>),
            RefCell::new(mesh_component),
            RefCell::new(transform_component),
        ];
        self.borrow_mut().create_object(object_id, components);

        self.borrow_mut().m_level_res_url = level_res_url.to_string();
        self.borrow_mut().m_is_loaded = true;
        info!("Level load succeed!");
        Ok(())
    }

    fn save(&self) -> Result<()> {
        info!("Saving level: {}", self.borrow().m_level_res_url);
        let mut output_level_res = LevelRes::default();

        self.borrow().m_entities.iter().for_each(|(object_id, entity)|{
            let mut output_object = ObjectInstanceRes::default();
            output_object.m_name = entity.borrow().get_name().to_string();
            output_object.m_definition = entity.borrow().get_definition_url().to_string();

            let borrowed_level = self.borrow();
            let index = borrowed_level.m_entity_location.get(object_id).unwrap();
            let components = 
                borrowed_level.m_archetypes.get(&index.0).unwrap().get_entity(index.1)
                .map(|component| {
                    component.borrow().clone_box()
                })
                .collect::<Vec<_>>();
            output_object.m_instanced_components = components;

            output_level_res.m_objects.push(output_object);
        });

        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        assert_manager.save_asset(&self.borrow().m_level_res_url, output_level_res)?;

        info!("Level save succeed!");
        Ok(())
    }

}