use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{editor::editor_global_context::{EditorGlobalContext, EditorGlobalContextCreateInfo}, runtime::{engine::{Engine, G_IS_EDITOR_MODE}, function::global::global_context::RuntimeGlobalContext}};




#[derive(Default)]
pub struct Editor {
    m_engine_runtime: Weak<RefCell<Engine>>,
}

impl Editor {
    pub fn initialize(&mut self, engine_runtime: &Rc<RefCell<Engine>>){
        unsafe{ G_IS_EDITOR_MODE = true; }
        self.m_engine_runtime = Rc::downgrade(engine_runtime);

        let info = EditorGlobalContextCreateInfo {
            window_system: &RuntimeGlobalContext::global().borrow().m_window_system,
            render_system: &RuntimeGlobalContext::global().borrow().m_render_system,
            engine_runtime: engine_runtime,
        };
        EditorGlobalContext::initialize(info);
        EditorGlobalContext::global().borrow().m_scene_manager.borrow_mut().set_editor_camera(
            RuntimeGlobalContext::global().borrow().m_render_system.borrow().get_render_camera()
        );
    }
}