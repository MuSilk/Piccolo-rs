use std::{path::Path, sync::{Arc, Mutex, MutexGuard, OnceLock}};

use crate::runtime::function::render::{debugdraw::debug_draw_manager::DebugDrawManager, render_system::RenderSystem, window_system::WindowSystem};

pub struct RuntimeGlobalContext {
    pub m_window_system: Arc<Mutex<WindowSystem>>,
    pub m_render_system: Arc<Mutex<RenderSystem>>,
    pub m_debugdraw_manager: Arc<Mutex<DebugDrawManager>>,
}

static RUNTIME_GLOBAL_CONTEXT: OnceLock<Mutex<RuntimeGlobalContext>> = OnceLock::new();

impl RuntimeGlobalContext {

    pub fn global() -> MutexGuard<'static, Self> {
        let ctx = RUNTIME_GLOBAL_CONTEXT.get().unwrap();
        ctx.lock().unwrap()
    }
    pub fn start_systems(&mut self, config_file_path: &Path){
        RUNTIME_GLOBAL_CONTEXT.get_or_init(|| Mutex::new(RuntimeGlobalContext {
            m_window_system: Arc::new(Mutex::new(WindowSystem::default())),
            m_render_system: Arc::new(Mutex::new(RenderSystem::default())),
            m_debugdraw_manager: Arc::new(Mutex::new(DebugDrawManager::default())),
        }));
    }


    fn shutdown_systems(&mut self){
    }
}