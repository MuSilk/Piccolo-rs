use crate::engine::Engine;

pub struct WindowUIInitInfo<'a> {
    pub engine: &'a Engine,
}

pub trait WindowUI {
    fn initialize(&mut self, init_info: WindowUIInitInfo);
    fn pre_render(&self, engine: &Engine);
}
