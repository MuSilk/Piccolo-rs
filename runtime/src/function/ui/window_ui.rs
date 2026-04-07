use std::{cell::RefCell, rc::Rc};

use crate::function::{input::input_system::InputSystem, render::{render_system::RenderSystem, window_system::WindowSystem}};

pub struct WindowUIInitInfo<'a> {
    pub window_system: &'a Rc<RefCell<WindowSystem>>,
    pub render_system: &'a Rc<RefCell<RenderSystem>>,
}

pub trait WindowUI {
    fn initialize(&mut self, init_info: WindowUIInitInfo);
    fn pre_render(&mut self, 
        render_system: &RefCell<RenderSystem>,
        window_system: &WindowSystem, 
        input_system: &RefCell<InputSystem>, 
        ui: &mut imgui::Ui
    );
}

