use std::{cell::{RefCell}, path::Path, rc::{Rc, Weak}};

use winit::event_loop::ActiveEventLoop;

use crate::{engine::Engine, function::{framework::world::world_manager::WorldManager, input::input_system::{InputSystem, InputSystemExt}, render::{debugdraw::debug_draw_manager::{DebugDrawManager, DebugDrawManagerCreateInfo}, render_system::{RenderSystem, RenderSystemCreateInfo}, window_system::{WindowCreateInfo, WindowSystem}}}, resource::{asset_manager::AssetManager, config_manager::ConfigManager}};

pub struct RuntimeGlobalContext {
    m_config_manager: Rc<RefCell<ConfigManager>>,
    m_asset_manager: Rc<RefCell<AssetManager>>,
    m_input_system: Rc<RefCell<InputSystem>>,
    m_world_manager: Rc<RefCell<WorldManager>>,
    m_window_system: Rc<RefCell<WindowSystem>>,
    m_render_system: Option<Rc<RefCell<RenderSystem>>>,
    m_debugdraw_manager: Option<Rc<RefCell<DebugDrawManager>>>,
}

impl RuntimeGlobalContext {
    pub fn new(config_file_path: &Path) -> Self {
        let config_manager = 
            Rc::new(RefCell::new(ConfigManager::default()));
        let asset_manager = 
            Rc::new(RefCell::new(AssetManager::new()));

        let ctx= RuntimeGlobalContext {
            m_config_manager: config_manager,
            m_asset_manager: asset_manager,
            m_input_system: Rc::new(RefCell::new(InputSystem::default())),
            m_world_manager: Rc::new(RefCell::new(WorldManager::default())),
            m_window_system: Rc::new(RefCell::new(WindowSystem::default())),
            m_render_system: None,
            m_debugdraw_manager: None,
        };
        ctx.m_config_manager.borrow_mut().initialize(config_file_path);
        ctx.m_world_manager.borrow_mut().initialize(
            &ctx.m_config_manager.borrow()
        );
        ctx
    }

    pub fn resumed_instance(&mut self, event_loop: &ActiveEventLoop, engine: Weak<RefCell<Engine>>) {
        self.m_window_system
            .borrow_mut()
            .initialize(event_loop, WindowCreateInfo::default())
            .unwrap();
        self.m_input_system
            .initialize(engine, &self.m_window_system);

        let render_system = RenderSystem::create(&RenderSystemCreateInfo {
            window_system: &self.m_window_system.borrow(),
            asset_manager: &self.m_asset_manager.borrow(),
            config_manager: &self.m_config_manager.borrow(),
        });
        let debugdraw_manager = DebugDrawManager::create(&DebugDrawManagerCreateInfo {
            rhi: render_system.get_rhi(),
            font_path: self.m_config_manager.borrow().get_editor_font_path(),
        })
        .unwrap();
        self.m_render_system = Some(Rc::new(RefCell::new(render_system)));
        self.m_debugdraw_manager = Some(Rc::new(RefCell::new(debugdraw_manager)));
    }

    pub fn input_system(&self) -> &Rc<RefCell<InputSystem>> {
        &self.m_input_system
    }

    pub fn window_system(&self) -> &Rc<RefCell<WindowSystem>> {
        &self.m_window_system
    }

    pub fn render_system(&self) -> &Rc<RefCell<RenderSystem>> {
        self.m_render_system.as_ref().unwrap()
    }

    pub fn debugdraw_manager(&self) -> &Rc<RefCell<DebugDrawManager>> {
        self.m_debugdraw_manager.as_ref().unwrap()
    }

    pub fn config_manager(&self) -> &Rc<RefCell<ConfigManager>> {
        &self.m_config_manager
    }

    pub fn asset_manager(&self) -> &Rc<RefCell<AssetManager>> {
        &self.m_asset_manager
    }

    pub fn world_manager(&self) -> &Rc<RefCell<WorldManager>> {
        &self.m_world_manager
    }

    pub fn shutdown_systems(&self){
        self.m_render_system.as_ref().unwrap().borrow().get_rhi().borrow().wait_idle().unwrap();
        self.m_debugdraw_manager
            .as_ref().unwrap().borrow_mut()
            .destroy(&self.m_render_system.as_ref().unwrap().borrow());
        self.m_render_system.as_ref().unwrap().borrow().destroy().unwrap();
    }
}