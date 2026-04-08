use std::{cell::RefCell, rc::Rc};

use crate::{engine::Engine, function::{input::input_system::InputSystem, render::{render_system::RenderSystem, window_system::WindowSystem}}};

pub struct WindowUIInitInfo<'a> {
    pub engine: &'a Rc<RefCell<Engine>>,
}

pub trait WindowUI {
    fn initialize(&mut self, init_info: WindowUIInitInfo);
    fn pre_render(&mut self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>, 
    );
}

