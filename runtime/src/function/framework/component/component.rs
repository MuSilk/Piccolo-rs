use std::{ cell::RefCell, rc::{Rc, Weak}};

use reflection::{reflection_derive::ReflectWhiteListFields};

use crate::function::framework::{level::level::Level, object::{object_id_allocator::GObjectID}};

pub trait ComponentTrait {
    fn post_load_resource(&mut self, parent_level: &Rc<RefCell<Level>>, parent_object: GObjectID) {
        self.get_component_mut().m_parent_level = Rc::downgrade(parent_level);
        self.get_component_mut().m_parent_object = parent_object;
    }
    fn tick(&mut self, _delta_time: f32) {}

    fn is_dirty(&self) -> bool {
        self.get_component().m_is_dirty
    }

    fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.get_component_mut().m_is_dirty = is_dirty;
    }

    fn get_component(&self) -> &Component;
    fn get_component_mut(&mut self) -> &mut Component;
}

#[derive(Clone, Default, ReflectWhiteListFields)]
pub struct Component {
    pub m_parent_object : GObjectID,
    pub m_parent_level: Weak<RefCell<Level>>,
    pub m_is_dirty: bool,
    pub m_is_scale_dirty: bool,
    pub m_tick_in_editor_mode: bool,
}