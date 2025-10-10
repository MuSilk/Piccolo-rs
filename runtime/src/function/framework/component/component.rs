use std::rc::Weak;

use reflection::reflection_derive::ReflectWhiteListFields;

use crate::{function::framework::object::object::GObject};

#[derive(Default,ReflectWhiteListFields)]
pub struct Component {
    m_parent_object : Weak<GObject>,
    m_is_dirty: bool,
    m_is_scale_dirty: bool,
    m_tick_in_editor_mode: bool,
}

impl Component {
    pub fn is_dirty(&self) -> bool {
        self.m_is_dirty
    }
    pub fn set_dirty_flag(&mut self, is_dirty: bool) {
        self.m_is_dirty = is_dirty;
    }
}