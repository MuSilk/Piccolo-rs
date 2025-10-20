use std::{cell::RefCell, rc::Rc};

use crate::function::render::{render_system::RenderSystem, window_system::WindowSystem};

pub struct WindowUIInitInfo<'a> {
    pub window_system: &'a Rc<RefCell<WindowSystem>>,
    pub render_system: &'a Rc<RefCell<RenderSystem>>,
}

pub trait WindowUI {
    fn initialize(&mut self, init_info: WindowUIInitInfo);
    // fn pre_render(&mut self);
}

