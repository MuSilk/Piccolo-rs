use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{function::framework::{level::level::Level, object::object_id_allocator::GObjectID}};

pub struct GObject {
    m_parent_level: Weak<RefCell<Level>>,
    m_id: GObjectID,
    m_name: String,
    m_definition_url: String,
}

impl GObject {
    pub fn get_id(&self) -> GObjectID {
        self.m_id
    }

    pub fn set_name(&mut self, name: &str) {
        self.m_name = name.to_string();
    }

    pub fn get_name(&self) -> &str {
        self.m_name.as_str()
    }

}

pub struct WrappedGObject(Rc<RefCell<GObject>>);

impl std::ops::Deref for WrappedGObject {
    type Target = Rc<RefCell<GObject>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for WrappedGObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl WrappedGObject {

    pub fn new(id: GObjectID, parent_level: &Rc<RefCell<Level>>) -> Self {
        Self(
            Rc::new(RefCell::new(GObject {
                m_parent_level: Rc::downgrade(parent_level),
                m_id: id,
                m_name: String::new(),
                m_definition_url: String::new(),
            }))
        )
    }
}