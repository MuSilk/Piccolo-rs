use std::{cell::RefCell, rc::{Rc, Weak}};

use runtime::{app, engine::{Engine}, function::{ui::window_ui::{WindowUI, WindowUIInitInfo}}};

use crate::{editor_global_context::{EditorGlobalContext, EditorGlobalContextCreateInfo}, editor_ui::EditorUI};



pub struct Editor {
    m_editor_ui: Rc<RefCell<EditorUI>>,
    m_editor_runtime: Option<EditorGlobalContext>,
    m_engine_runtime: Weak<RefCell<Engine>>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            m_editor_ui: Rc::new(RefCell::new(EditorUI::default())),
            m_editor_runtime: None,
            m_engine_runtime: Weak::new(),
        }
    }
}

impl app::System for Editor {
    fn initialize(&mut self, engine_runtime: &Rc<RefCell<Engine>>){
        engine_runtime.borrow_mut().set_editor_mode(true);
        self.m_engine_runtime = Rc::downgrade(engine_runtime);

        let t_engine_runtime = engine_runtime.borrow();
        let render_system = 
            t_engine_runtime.m_runtime_context.render_system();
        let window_system = 
            t_engine_runtime.m_runtime_context.window_system();

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
            engine_runtime,
        );

        self.m_editor_ui.borrow_mut().initialize(WindowUIInitInfo{
            window_system: &window_system,
            render_system: &render_system,
        });

        let window_ui: Rc<RefCell<dyn WindowUI>> = self.m_editor_ui.clone();
        render_system.borrow_mut().initialize_ui_render_backend(&window_ui);
    }

    fn tick(&mut self, _delta_time: f32) {
        self.m_editor_runtime.as_ref().unwrap().m_input_manager.borrow().tick(
            &self.m_editor_runtime.as_ref().unwrap().m_scene_manager.borrow()
        );
    }
}