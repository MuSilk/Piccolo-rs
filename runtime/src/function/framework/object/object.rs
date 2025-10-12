use reflection::reflection::ReflectionPtr;

use crate::function::framework::{component::component::ComponentTrait, object::object_id_allocator::GObjectID};

pub struct GObject {
    m_id: GObjectID,
    m_name: String,
    m_definition_url: String,
    m_components: Vec<ReflectionPtr<Box<dyn ComponentTrait>>>,
}

impl GObject {

    pub fn get_id(&self) -> GObjectID {
        self.m_id
    }
    pub fn try_get_component<T: ComponentTrait>(&self, component_type_name: &str) -> Option<&mut Box<T>> {
        for component in &self.m_components {
            if component.get_type_name() == component_type_name {
                let res = component.get_ptr() as *mut Box<T>;
                return Some(unsafe { &mut *res });
            }
        }
        None
    }
}
