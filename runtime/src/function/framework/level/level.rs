use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{function::{framework::{component::{component::ComponentTrait, mesh::mesh_component::{MeshComponent}, transform::transform_component::TransformComponent}, object::{object::{WrappedGObject}, object_id_allocator::{self, GObjectID}}}}, resource::res_type::{common::object::ObjectInstanceRes, components::mesh::{SubMeshRes}}};

#[derive(Default)]
struct ComponentTable {
    m_components: Vec<Box<RefCell<dyn Any>>>,
    m_entity_to_index: HashMap<GObjectID, usize>,
    m_index_to_entity: Vec<GObjectID>,
}

impl ComponentTable {
    fn insert(&mut self, object_id: GObjectID, component: Box<RefCell<dyn Any>>){
        if let Some(&index) = self.m_entity_to_index.get(&object_id) {
            self.m_components[index] = component;
        }
        else {
            let index = self.m_index_to_entity.len();
            self.m_entity_to_index.insert(object_id, index);
            self.m_index_to_entity.push(object_id);
            self.m_components.push(component);
        }
    }

    fn get(&self, object_id: GObjectID) -> Option<&Box<RefCell<dyn Any>>> {
        if let Some(&index) = self.m_entity_to_index.get(&object_id) {
            return Some(&self.m_components[index]);
        }
        None
    }

}

pub type LevelObjectMap = HashMap<GObjectID, WrappedGObject>;

#[derive(Default)]
pub struct Level {
    m_is_loaded: bool,
    m_gobjects: LevelObjectMap,
    m_components: HashMap<TypeId, ComponentTable>,
}

impl Level {
    pub fn new() -> Rc<RefCell<Self>> {
        let level = Self::default();
        let level = Rc::new(RefCell::new(level));
        level
    }
    
    pub fn tick(&self, delta_time: f32) {
        if !self.m_is_loaded {
            return;
        }
        for (_, mut mesh_component) in self.query_mut::<MeshComponent>() {
            mesh_component.tick(delta_time);
        }
    }

    fn table_for<T: 'static + ComponentTrait>(&mut self) -> &mut ComponentTable {
        self.m_components.entry(TypeId::of::<T>()).or_insert_with(ComponentTable::default)
    }

    pub fn insert<C: 'static + ComponentTrait>(&mut self, object_id: GObjectID, component: Box<RefCell<C>>) {
        let table = self.table_for::<C>();
        table.insert(object_id, component);
    }

    pub fn get_component<C: 'static + ComponentTrait>(&self, object_id: GObjectID) -> Option<std::cell::Ref<'_, C>> {
        self.m_components
            .get(&TypeId::of::<C>())
            .and_then(|table| table.get(object_id))
            .and_then(|boxed| {
                let borrowed = boxed.as_ref().borrow();
                if borrowed.is::<C>() {
                    Some(std::cell::Ref::map(borrowed, |b| b.downcast_ref::<C>().unwrap()))
                } else {
                    None
                }
            }
        )
    }

    pub fn get_component_mut<C: 'static + ComponentTrait>(&self, object_id: GObjectID) -> Option<std::cell::RefMut<'_, C>> {
        self.m_components
            .get(&TypeId::of::<C>())
            .and_then(|table| table.get(object_id))
            .and_then(|boxed| {
                let borrowed = boxed.as_ref().borrow_mut();
                if borrowed.is::<C>() {
                    Some(std::cell::RefMut::map(borrowed, |b| b.downcast_mut::<C>().unwrap()))
                } else {
                    None
                }
            }
        )
    }

    pub fn query<C: 'static + ComponentTrait>(&self) -> Vec<(GObjectID, std::cell::Ref<'_, C>)> {
        let mut result = Vec::new();
        if let Some(table) = self.m_components.get(&TypeId::of::<C>()) {
            for &object_id in &table.m_index_to_entity {
                if let Some(component) = table.get(object_id).and_then(|boxed| {
                    Some(std::cell::Ref::map(boxed.borrow(), |b| b.downcast_ref::<C>().unwrap()))
                }) {
                    result.push((self.m_gobjects.get(&object_id).unwrap().borrow().get_id(), component));
                }
            }
        }
        result
    }

    pub fn query_mut<C: 'static + ComponentTrait>(&self) -> Vec<(GObjectID, std::cell::RefMut<'_, C>)> {
        let mut result = Vec::new();
        if let Some(table) = self.m_components.get(&TypeId::of::<C>()) {
            for &object_id in &table.m_index_to_entity {
                if let Some(component) = table.get(object_id).and_then(|boxed| {
                    Some(std::cell::RefMut::map(boxed.borrow_mut(), |b| b.downcast_mut::<C>().unwrap()))
                }) {
                    result.push((self.m_gobjects.get(&object_id).unwrap().borrow().get_id(), component));
                }
            }
        }
        result
    }
}

pub trait LevelExt {
    fn spawn(&self) -> GObjectID;
    fn load(&self);
}

impl LevelExt for Rc<RefCell<Level>> {
    fn spawn(&self) -> GObjectID {
        let object_id = object_id_allocator::alloc();
        let gobject = WrappedGObject::new(object_id, &self);
        self.borrow_mut().m_gobjects.insert(object_id, gobject);
        object_id
    }

    fn load(&self) {
        let object_id = self.spawn();
        let mesh_component = Box::new(RefCell::new(MeshComponent::default()));
        mesh_component.borrow_mut().m_mesh_res.m_sub_meshs.push(SubMeshRes{
            m_obj_file_ref: "asset/bunny_200.obj".to_string(),
            ..Default::default()
        });
        mesh_component.borrow_mut().post_load_resource(&self,  object_id);
        let transform_component = Box::new(RefCell::new(TransformComponent::default()));
        transform_component.borrow_mut().post_load_resource(&self, object_id);
        self.borrow_mut().insert(object_id, mesh_component);
        self.borrow_mut().insert(object_id, transform_component);
        self.borrow_mut().m_is_loaded = true;
    }
}