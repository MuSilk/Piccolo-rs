use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use crate::function::framework::{
    component::component::ComponentTrait, object::object_id_allocator::GObjectID,
};

pub struct GObject {
    m_id: GObjectID,
    m_name: String,
    m_definition_url: String,
    pub m_components: HashMap<TypeId, RefCell<Box<dyn ComponentTrait>>>,
}

impl GObject {
    pub fn new(id: GObjectID) -> GObject {
        GObject {
            m_id: id,
            m_name: String::new(),
            m_definition_url: String::new(),
            m_components: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> GObjectID {
        self.m_id
    }

    pub fn set_name(&mut self, name: &str) {
        self.m_name = name.to_string();
    }

    pub fn get_name(&self) -> &str {
        self.m_name.as_str()
    }

    pub fn set_definition_url(&mut self, url: &str) {
        self.m_definition_url = url.to_string();
    }

    pub fn get_definition_url(&self) -> &str {
        self.m_definition_url.as_str()
    }

    pub fn get_component<T: 'static + ComponentTrait>(&self) -> Option<Ref<'_, T>> {
        let component = self.m_components.get(&TypeId::of::<T>())?;
        Ref::filter_map(component.borrow(), |component| {
            component.as_any().downcast_ref::<T>()
        })
        .ok()
    }

    pub fn get_component_mut<T: 'static + ComponentTrait>(&self) -> Option<RefMut<'_, T>> {
        let component = self.m_components.get(&TypeId::of::<T>())?;
        RefMut::filter_map(component.borrow_mut(), |component| {
            component.as_any_mut().downcast_mut::<T>()
        })
        .ok()
    }
}
