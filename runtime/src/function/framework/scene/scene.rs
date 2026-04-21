use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
};

use crate::{
    engine::Engine,
    function::framework::{
        component::{component::ComponentTrait, transform_component::TransformComponent},
        object::{
            object::GObject,
            object_id_allocator::{self, GObjectID},
        },
    },
};

#[derive(Default)]
pub struct Scene {
    m_is_loaded: bool,
    m_level_res_url: String,
    pub m_entities: HashMap<GObjectID, GObject>,
    m_resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Scene {
    fn spawn(&mut self) -> GObjectID {
        let object_id = object_id_allocator::alloc();
        let gobject = GObject::new(object_id);
        self.m_entities.insert(object_id, gobject);
        object_id
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_url(&mut self, level_res_url: &str) {
        self.m_level_res_url = level_res_url.to_string();
    }

    pub fn get_url(&self) -> String {
        self.m_level_res_url.clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.m_is_loaded
    }

    pub fn set_loaded(&mut self, loaded: bool) {
        self.m_is_loaded = loaded;
    }

    pub fn create_object(
        &mut self,
        components: Vec<RefCell<Box<dyn ComponentTrait>>>,
    ) -> GObjectID {
        let object_id = self.spawn();
        let gobject = self.m_entities.get_mut(&object_id).unwrap();
        for component in components {
            let component_type_id = {
                let borrowed = component.borrow();
                borrowed.as_any().type_id()
            };
            gobject
                .m_components
                .insert(component_type_id, component);
        }
        object_id
    }

    pub fn tick(&mut self, engine: &Engine, delta_time: f32) {
        let transform_type_id = TypeId::of::<TransformComponent>();

        // Ensure transform buffers are updated first for every entity.
        // Mesh and other render-related components read transform.current in the same frame.
        for (_, gobject) in self.m_entities.iter_mut() {
            if let Some(component) = gobject.m_components.get(&transform_type_id) {
                component
                    .borrow_mut()
                    .tick(engine, &gobject, delta_time);
            }
        }

        for (_, gobject) in self.m_entities.iter_mut() {
            let component_ids = {
                gobject.m_components.keys().copied().collect::<Vec<_>>()
            };
            for component_id in component_ids {
                if component_id == transform_type_id {
                    continue;
                }
                if let Some(component) = gobject.m_components.get(&component_id) {
                    component
                        .borrow_mut()
                        .tick(engine, &gobject, delta_time);
                }
            }
        }
    }

    pub fn delete_object_by_id(&mut self, engine: &Engine, object_id: GObjectID) {
        let mut gobject = self.m_entities.remove(&object_id).unwrap();
        for (_, component) in gobject.m_components.iter_mut() {
            component.borrow_mut().on_delete(engine);
        }
    }

    pub fn add_resource<T: 'static>(&mut self, resource: T) {
        self.m_resources
            .insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.m_resources
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    pub fn get_mut_resource<T: 'static>(&mut self) -> Option<&mut T> {
        self.m_resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    pub fn get_gobject_by_id(&self, object_id: GObjectID) -> Option<&GObject> {
        self.m_entities.get(&object_id)
    }
}

pub trait SceneTrait {
    fn load(&mut self, engine: &Engine);
    fn save(&self);
    fn tick(&mut self, engine_runtime: &Engine, delta_time: f32);
    fn get_url(&self) -> String;
    fn is_loaded(&self) -> bool;
}
