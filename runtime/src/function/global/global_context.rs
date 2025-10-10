use std::{cell::{RefCell}, path::Path, rc::Rc};

use anyhow::Result;
use winit::event_loop::ActiveEventLoop;

use crate::{function::render::{debugdraw::debug_draw_manager::{DebugDrawManager, DebugDrawManagerCreateInfo}, render_system::{RenderSystem, RenderSystemCreateInfo}, window_system::{WindowCreateInfo, WindowSystem}}, resource::config_manager::ConfigManager};

pub struct RuntimeGlobalContext {
    pub m_config_manager: Rc<RefCell<ConfigManager>>,
    pub m_window_system: Rc<RefCell<WindowSystem>>,
    pub m_render_system: Rc<RefCell<RenderSystem>>,
    pub m_debugdraw_manager: Rc<RefCell<DebugDrawManager>>,
}

static mut G_RUNTIME_GLOBAL_CONTEXT: Option<RefCell<RuntimeGlobalContext>> = None;

unsafe impl Send for RuntimeGlobalContext {}
unsafe impl Sync for RuntimeGlobalContext {}

impl RuntimeGlobalContext {

    pub fn isinitialized() -> bool {
        unsafe{
            #[allow(static_mut_refs)]
            G_RUNTIME_GLOBAL_CONTEXT.is_some()
        }
    }

    pub fn global() -> &'static RefCell<Self> {
        unsafe{
            #[allow(static_mut_refs)]
            G_RUNTIME_GLOBAL_CONTEXT.as_ref().unwrap()
        }   
    }
    pub fn start_systems(event_loop: &ActiveEventLoop, config_file_path: &Path) -> Result<()> {
        let mut config_manager = ConfigManager::default();
        config_manager.initialize(config_file_path);

        let mut window_system = WindowSystem::default();
        window_system.initialize(event_loop, WindowCreateInfo::default())?;

        let render_system = RenderSystem::create(&RenderSystemCreateInfo {
            window_system: &window_system
        })?;

        let debugdraw_manager = DebugDrawManager::create(&DebugDrawManagerCreateInfo {
            rhi: render_system.get_rhi(),
            font_path: config_manager.get_editor_font_path(),
        })?;

        unsafe{
            G_RUNTIME_GLOBAL_CONTEXT = Some(RefCell::new(RuntimeGlobalContext {
                m_config_manager: Rc::new(RefCell::new(config_manager)),
                m_window_system: Rc::new(RefCell::new(window_system)),
                m_render_system: Rc::new(RefCell::new(render_system)),
                m_debugdraw_manager: Rc::new(RefCell::new(debugdraw_manager))
            }));
        }
        Ok(())
    }


    pub fn shutdown_systems(&self){
        self.m_render_system.borrow().get_rhi().borrow().wait_idle().unwrap();
        self.m_debugdraw_manager.borrow_mut().destroy();
        self.m_render_system.borrow().destroy().unwrap();
    }
}