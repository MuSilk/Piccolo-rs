use std::{cell::{OnceCell, RefCell}, path::Path, rc::Rc, sync::{Mutex, MutexGuard, OnceLock}};

use crate::runtime::function::render::{debugdraw::debug_draw_manager::DebugDrawManager, render_system::RenderSystem, window_system::WindowSystem};

pub struct RuntimeGlobalContext {
    pub m_window_system: Rc<RefCell<WindowSystem>>,
    pub m_render_system: Rc<RefCell<RenderSystem>>,
    pub m_debugdraw_manager: Rc<RefCell<DebugDrawManager>>,
}

static mut RUNTIME_GLOBAL_CONTEXT: Option<RefCell<RuntimeGlobalContext>> = None;

unsafe impl Send for RuntimeGlobalContext {}
unsafe impl Sync for RuntimeGlobalContext {}

impl RuntimeGlobalContext {

    pub fn isinitialized() -> bool {
        unsafe{
            #[allow(static_mut_refs)]
            RUNTIME_GLOBAL_CONTEXT.is_some()
        }
    }

    pub fn global() -> &'static RefCell<Self> {
        unsafe{
            #[allow(static_mut_refs)]
            RUNTIME_GLOBAL_CONTEXT.as_ref().unwrap()
        }   
    }
    pub fn start_systems(config_file_path: &Path){
        unsafe{
            RUNTIME_GLOBAL_CONTEXT = Some(RefCell::new(RuntimeGlobalContext {
                m_window_system: Rc::new(RefCell::new(WindowSystem::default())),
                m_render_system: Rc::new(RefCell::new(RenderSystem::default())),
                m_debugdraw_manager: Rc::new(RefCell::new(DebugDrawManager::default())),
            }));
        }
    }


    fn shutdown_systems(&mut self){
        
    }
}