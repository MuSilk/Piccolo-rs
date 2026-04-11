use std::{cell::RefCell, rc::Rc};

use crate::{engine::Engine};

pub struct WindowUIInitInfo<'a> {
    pub engine: &'a Rc<RefCell<Engine>>,
}

pub trait WindowUI {
    fn initialize(&mut self, init_info: WindowUIInitInfo);
    fn pre_render(&self);
}

