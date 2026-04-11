use std::{cell::{RefCell}, rc::{Rc}};

use runtime::{engine::Engine};

use crate::{editor_input_manager::{EditorInputManager, EditorInputManagerExt}, editor_scene_manager::EditorSceneManager};

pub struct EditorGlobalContextCreateInfo<'a> {
    pub engine_runtime: &'a Rc<RefCell<Engine>>,
}

pub struct EditorGlobalContext {
    pub m_scene_manager: Rc<RefCell<EditorSceneManager>>,
    pub m_input_manager: Rc<RefCell<EditorInputManager>>,
}

impl EditorGlobalContext {

    pub fn new(init_info: EditorGlobalContextCreateInfo) -> Self {
        let ctx = EditorGlobalContext {
            m_scene_manager: Rc::new(RefCell::new(EditorSceneManager::default())),
            m_input_manager: Rc::new(RefCell::new(EditorInputManager::default())),
        };
        ctx.m_scene_manager.borrow_mut().initialize();
        ctx.m_input_manager
            .initialize(init_info.engine_runtime, &ctx.m_scene_manager);
        ctx
    }
}