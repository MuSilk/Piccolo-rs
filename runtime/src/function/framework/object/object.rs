use std::{cell::RefCell, rc::Rc};

use crate::{function::framework::{object::object_id_allocator::GObjectID}};

pub struct GObject {
    m_id: GObjectID,
    m_name: String,
    m_definition_url: String,
}

impl GObject {

    pub fn new(id: GObjectID) -> Rc<RefCell<GObject>> {
        Rc::new(RefCell::new(GObject {
            m_id: id,
            m_name: String::new(),
            m_definition_url: String::new(),
        }))
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
}