use std::{cell::RefCell, path::Path, rc::Rc};

use winit::event_loop::ActiveEventLoop;

use crate::{
    function::{
        framework::world::world_manager::WorldManager,
        input::{game_command_system::GameCommandInputSystem, input_system::InputSystem},
        render::{
            render_system::{RenderSystem, RenderSystemCreateInfo},
            window_system::{WindowCreateInfo, WindowSystem},
        },
        ui::ui2::UiRuntime,
    },
    resource::{asset_manager::AssetManager, config_manager::ConfigManager},
};

pub struct RuntimeGlobalContext {
    m_config_manager: ConfigManager,
    m_asset_manager: AssetManager,
    m_input_system: RefCell<GameCommandInputSystem>,
    m_world_manager: RefCell<WorldManager>,
    m_window_system: Rc<RefCell<WindowSystem>>,
    m_render_system: Option<RefCell<RenderSystem>>,
    m_ui_runtime: RefCell<UiRuntime>,
}

impl RuntimeGlobalContext {
    pub fn new(config_file_path: &Path) -> Self {
        let mut config_manager = ConfigManager::default();
        config_manager.initialize(config_file_path);
        let asset_manager = AssetManager::new(&config_manager);

        let ctx = RuntimeGlobalContext {
            m_config_manager: config_manager,
            m_asset_manager: asset_manager,
            m_input_system: RefCell::new(GameCommandInputSystem::default()),
            m_world_manager: RefCell::new(WorldManager::default()),
            m_window_system: Rc::new(RefCell::new(WindowSystem::default())),
            m_render_system: None,
            m_ui_runtime: RefCell::new(UiRuntime::default()),
        };
        ctx.m_world_manager
            .borrow_mut()
            .initialize(&ctx.m_config_manager);
        ctx
    }

    pub fn resumed_instance(&mut self, event_loop: &ActiveEventLoop) {
        self.m_window_system
            .borrow_mut()
            .initialize(event_loop, WindowCreateInfo::default())
            .unwrap();

        self.register_input_system();

        let render_system = RenderSystem::create(&RenderSystemCreateInfo {
            window_system: &self.m_window_system.borrow(),
            asset_manager: &self.m_asset_manager,
            config_manager: &self.m_config_manager,
        });
        self.m_ui_runtime
            .borrow_mut()
            .load_font_texture(&self.m_config_manager)
            .unwrap();
        self.m_render_system = Some(RefCell::new(render_system));
    }

    pub fn input_system(&self) -> &RefCell<GameCommandInputSystem> {
        &self.m_input_system
    }

    pub fn window_system(&self) -> &Rc<RefCell<WindowSystem>> {
        &self.m_window_system
    }

    pub fn render_system(&self) -> &RefCell<RenderSystem> {
        self.m_render_system.as_ref().unwrap()
    }

    pub fn config_manager(&self) -> &ConfigManager {
        &self.m_config_manager
    }

    pub fn asset_manager(&self) -> &AssetManager {
        &self.m_asset_manager
    }

    pub fn world_manager(&self) -> &RefCell<WorldManager> {
        &self.m_world_manager
    }

    pub fn ui_runtime(&self) -> &RefCell<UiRuntime> {
        &self.m_ui_runtime
    }

    pub fn shutdown_systems(&self) {
        self.m_ui_runtime.borrow_mut().destroy_textures();
        self.m_render_system
            .as_ref()
            .unwrap()
            .borrow()
            .get_rhi()
            .borrow()
            .wait_idle()
            .unwrap();
        self.m_render_system
            .as_ref()
            .unwrap()
            .borrow()
            .destroy()
            .unwrap();
    }
}

impl RuntimeGlobalContext {
    fn register_input_system(&self) {
        let mut window_system = self.window_system().borrow_mut();

        window_system.register_on_key_func(move |engine, device_id, event, is_synthetic| {
            engine
                .input_system()
                .borrow_mut()
                .on_key(engine, device_id, event, is_synthetic);
        });

        window_system.register_on_mouse_motion(move |engine, device_id, position| {
            engine
                .input_system()
                .borrow_mut()
                .on_mouse_motion(engine, device_id, position);
        });

        window_system.register_on_cursor_pos_func(move |engine, device_id, position| {
            engine
                .input_system()
                .borrow_mut()
                .on_cursor_pos(device_id, position);
        });

        window_system.register_on_mouse_button_func(move |engine, device_id, state, button| {
            engine
                .input_system()
                .borrow_mut()
                .on_mouse_button(device_id, state, button);
        });
    }
}
