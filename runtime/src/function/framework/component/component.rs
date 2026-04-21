use std::any::Any;

use crate::{engine::Engine, function::framework::object::object::GObject};

pub trait ComponentTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn on_delete(&mut self, _engine: &Engine) {}
    fn tick(&mut self, _engine: &Engine, _gobject: &GObject, _delta_time: f32) {}
}
