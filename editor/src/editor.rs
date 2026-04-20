use std::{cell::RefCell, rc::{Rc}};

use runtime::{engine::{self, Engine}, function::ui::window_ui::{WindowUI, WindowUIInitInfo}};

use crate::{editor_global_context::{EditorGlobalContext, EditorGlobalContextCreateInfo}, editor_ui::EditorUI};



pub struct Editor {
    m_editor_ui: Rc<RefCell<EditorUI>>,
    m_editor_runtime: Option<EditorGlobalContext>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            m_editor_ui: Rc::new(RefCell::new(EditorUI::default())),
            m_editor_runtime: None,
        }
    }
}

impl engine::System for Editor {
    fn initialize(&mut self, engine_runtime: &Engine){
        engine_runtime.set_editor_mode(true);

        let render_system = 
            engine_runtime.render_system();

        let info = EditorGlobalContextCreateInfo {
            engine_runtime: engine_runtime,
        };
        self.m_editor_runtime = Some(EditorGlobalContext::new(info));
        let editor_ctx = self.m_editor_runtime.as_ref().unwrap();
        editor_ctx.m_scene_manager.borrow_mut().set_editor_camera(
            render_system.borrow().get_render_camera()
        );

        self.m_editor_ui.borrow_mut().set_editor_handles(
            editor_ctx.m_input_manager.clone(),
            editor_ctx.m_scene_manager.clone(),
        );

        self.m_editor_ui.borrow_mut().initialize(WindowUIInitInfo{
            engine: engine_runtime,
        });
    }

    fn tick(&mut self, engine: &Engine, _delta_time: f32) {
        self.m_editor_runtime.as_ref().unwrap().m_input_manager.borrow().tick(
            &self.m_editor_runtime.as_ref().unwrap().m_scene_manager.borrow()
        );
        self.m_editor_ui.borrow().pre_render(engine);
    }
}