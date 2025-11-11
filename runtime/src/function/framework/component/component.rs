use std::{any::{Any}};

use crate::function::framework::{object::object_id_allocator::GObjectID};

pub trait ComponentTrait {

    fn is_dirty(&self) -> bool {
        self.get_component().m_is_dirty
    }

    fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.get_component_mut().m_is_dirty = is_dirty;
    }

    fn get_component(&self) -> &Component;
    fn get_component_mut(&mut self) -> &mut Component;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn ComponentTrait>;
}

#[derive(Clone, Default)]
pub struct Component {
    pub m_parent_object : GObjectID,
    pub m_is_dirty: bool,
    pub m_is_scale_dirty: bool,
    pub m_tick_in_editor_mode: bool,
}