use std::{cell::{RefCell}, path::Path, rc::Rc};

use anyhow::Result;
use winit::event_loop::ActiveEventLoop;

use crate::{function::{framework::world::world_manager::WorldManager, render::{debugdraw::debug_draw_manager::{DebugDrawManager, DebugDrawManagerCreateInfo}, render_system::{RenderSystem, RenderSystemCreateInfo}, window_system::{WindowCreateInfo, WindowSystem}}}, resource::{asset_manager::AssetManager, config_manager::ConfigManager}};

#[derive(Default)]
pub struct RuntimeGlobalContext {
    pub m_asset_manager: Rc<RefCell<AssetManager>>,
    pub m_config_manager: Rc<RefCell<ConfigManager>>,
    pub m_world_manager: Rc<RefCell<WorldManager>>,
    m_window_system: Rc<RefCell<WindowSystem>>,
    m_render_system: Option<Rc<RefCell<RenderSystem>>>,
    m_debugdraw_manager: Option<Rc<RefCell<DebugDrawManager>>>,
}

static mut G_RUNTIME_GLOBAL_CONTEXT: Option<RuntimeGlobalContext> = None;

unsafe impl Send for RuntimeGlobalContext {}
unsafe impl Sync for RuntimeGlobalContext {}

impl RuntimeGlobalContext {
    #[allow(static_mut_refs)]
    pub fn global() -> &'static Self {
        unsafe{
            G_RUNTIME_GLOBAL_CONTEXT.as_ref().unwrap()
        }   
    }
    
    #[allow(static_mut_refs)]
    pub fn start_systems(event_loop: &ActiveEventLoop, config_file_path: &Path) -> Result<()> {
        unsafe{
            G_RUNTIME_GLOBAL_CONTEXT = Some(RuntimeGlobalContext::default());
            let ctx = G_RUNTIME_GLOBAL_CONTEXT.as_ref().unwrap();
            ctx.m_config_manager.borrow_mut().initialize(config_file_path);
            ctx.m_world_manager.borrow_mut().initialize(&ctx.m_config_manager.borrow().get_default_world_url());
            ctx.m_window_system.borrow_mut().initialize(event_loop, WindowCreateInfo::default())?;

            let render_system = RenderSystem::create(&RenderSystemCreateInfo {
                window_system: &ctx.m_window_system.borrow(),
            })?;
            let debugdraw_manager = DebugDrawManager::create(&DebugDrawManagerCreateInfo {
                rhi: render_system.get_rhi(),
                font_path: ctx.m_config_manager.borrow().get_editor_font_path(),
            })?;
            G_RUNTIME_GLOBAL_CONTEXT.as_mut().unwrap().m_render_system = Some(Rc::new(RefCell::new(render_system)));
            G_RUNTIME_GLOBAL_CONTEXT.as_mut().unwrap().m_debugdraw_manager = Some(Rc::new(RefCell::new(debugdraw_manager)));
        }
        Ok(())
    }


    pub fn shutdown_systems(&self){
        self.m_render_system.as_ref().unwrap().borrow().get_rhi().borrow().wait_idle().unwrap();
        self.m_debugdraw_manager.as_ref().unwrap().borrow_mut().destroy();
        self.m_render_system.as_ref().unwrap().borrow().destroy().unwrap();
    }

    pub fn get_window_system() -> &'static Rc<RefCell<WindowSystem>> {
        &Self::global().m_window_system
    }

    pub fn get_render_system() -> &'static Rc<RefCell<RenderSystem>> {
        &Self::global().m_render_system.as_ref().unwrap()
    }

    pub fn get_debugdraw_manager() -> &'static Rc<RefCell<DebugDrawManager>> {
        &Self::global().m_debugdraw_manager.as_ref().unwrap()
    }
}