use crate::runtime::function::render::{render_system::RenderSystem, window_system::WindowSystem};

pub struct WindowUIInitInfo<'a>{
    pub window_system: &'a WindowSystem,
    pub render_system: &'a RenderSystem,
}

pub trait WindowUI {
    fn initialize(&self, info: WindowUIInitInfo);
    fn pre_render(&self);
}