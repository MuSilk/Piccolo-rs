use std::{cell::RefCell, rc::{Rc, Weak}};

use runtime::{app, engine::{Engine, G_IS_EDITOR_MODE}, function::{global::global_context::RuntimeGlobalContext, ui::window_ui::{WindowUI, WindowUIInitInfo}}};

use crate::{editor_global_context::{EditorGlobalContext, EditorGlobalContextCreateInfo}, editor_input_manager::EditorInputManagerExt, editor_ui::EditorUI};



pub struct Editor {
    m_editor_ui: Rc<RefCell<dyn WindowUI>>,
    m_engine_runtime: Weak<RefCell<Engine>>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            m_editor_ui: Rc::new(RefCell::new(EditorUI::default())),
            m_engine_runtime: Weak::new(),
        }
    }
}

impl app::System for Editor {
    fn initialize(&mut self, engine_runtime: &Rc<RefCell<Engine>>){
        unsafe{ G_IS_EDITOR_MODE = true; }
        self.m_engine_runtime = Rc::downgrade(engine_runtime);

        let info = EditorGlobalContextCreateInfo {
            window_system: &RuntimeGlobalContext::get_window_system(),
            render_system: &RuntimeGlobalContext::get_render_system(),
            engine_runtime: engine_runtime,
        };
        EditorGlobalContext::initialize(info);
        EditorGlobalContext::global().borrow().m_scene_manager.borrow_mut().set_editor_camera(
            RuntimeGlobalContext::get_render_system().borrow().get_render_camera()
        );

        self.m_editor_ui.borrow_mut().initialize(WindowUIInitInfo{
            window_system: &RuntimeGlobalContext::get_window_system(),
            render_system: &RuntimeGlobalContext::get_render_system(),
        });

        RuntimeGlobalContext::get_render_system().borrow_mut().initialize_ui_render_backend(&self.m_editor_ui);
    }

    fn tick(&mut self, delta_time: f32) {
        EditorGlobalContext::global().borrow().m_input_manager.tick(delta_time);
    }
}