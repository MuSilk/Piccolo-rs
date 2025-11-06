use std::{cell::{RefCell}, rc::{Rc, Weak}};

use runtime::{engine::Engine, function::render::{render_system::RenderSystem, window_system::WindowSystem}};

use crate::{editor_input_manager::{EditorInputManager, EditorInputManagerExt}, editor_scene_manager::EditorSceneManager};

static mut G_EDITOR_GLOBAL_CONTEXT: Option<RefCell<EditorGlobalContext>> = None;

pub struct EditorGlobalContextCreateInfo<'a> {
    pub window_system: &'a Rc<RefCell<WindowSystem>>,
    pub render_system: &'a Rc<RefCell<RenderSystem>>,
    pub engine_runtime: &'a Rc<RefCell<Engine>>,
}

pub struct EditorGlobalContext {
    pub m_scene_manager: Rc<RefCell<EditorSceneManager>>,
    pub m_input_manager: Rc<RefCell<EditorInputManager>>,
    pub m_render_system: Weak<RefCell<RenderSystem>>,
    pub m_window_system: Weak<RefCell<WindowSystem>>,
    m_engine_runtime: Weak<RefCell<Engine>>,
}

impl EditorGlobalContext {

    pub fn global() -> &'static RefCell<Self> {
        unsafe{
            #[allow(static_mut_refs)]
            G_EDITOR_GLOBAL_CONTEXT.as_ref().unwrap()
        }
    }

    pub fn initialize(init_info: EditorGlobalContextCreateInfo){
        unsafe{
            let ctx = EditorGlobalContext {
                m_scene_manager: Rc::new(RefCell::new(EditorSceneManager::default())),
                m_input_manager: Rc::new(RefCell::new(EditorInputManager::default())),
                m_render_system: Rc::downgrade(init_info.render_system),
                m_window_system: Rc::downgrade(init_info.window_system),
                m_engine_runtime: Rc::downgrade(init_info.engine_runtime),
            };
            ctx.m_scene_manager.borrow_mut().initialize();
            ctx.m_input_manager.initialize();
            G_EDITOR_GLOBAL_CONTEXT = Some(RefCell::new(ctx));
            
        }
    }
}