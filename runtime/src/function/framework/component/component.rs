use std::{any::{Any}, cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::function::framework::{level::level::Level, object::object_id_allocator::GObjectID};

#[typetag::serde(tag = "$type_name")]
pub trait ComponentTrait {
    fn post_load_resource(&mut self, _parent_level: &Rc<RefCell<Level>>, parent_object: GObjectID) {
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

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Component {
    pub m_parent_object : GObjectID,
    pub m_is_dirty: bool,
    pub m_is_scale_dirty: bool,
    pub m_tick_in_editor_mode: bool,
}