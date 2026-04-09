use std::{cell::RefCell, path::Path, rc::{Rc, Weak}, time::Instant};

use anyhow::Result;
use winit::event_loop::ActiveEventLoop;

use crate::{function::{framework::world::world_manager::WorldManager, global::global_context::RuntimeGlobalContext, input::input_system::InputSystem, render::{render_system::RenderSystem, window_system::WindowSystem}, ui::ui2::UiRuntime}, resource::{asset_manager::AssetManager, config_manager::ConfigManager}};

const S_FPS_ALPHA: f32 = 1.0 / 100.0;

pub struct Engine {
    m_runtime_context: RuntimeGlobalContext,
    m_is_quit: bool,
    m_state: RefCell<EngineState>,
}

struct EngineState {
    m_last_tick_time_point: Instant,
    m_frame_count: u32,
    m_average_duration: f32,
    m_fps: u32,
    m_is_editor_mode: bool,
}

impl Engine {

    pub fn new(config_file_path: &Path) -> Self {
        Engine {
            m_runtime_context: RuntimeGlobalContext::new(config_file_path),
            m_is_quit: false,
            m_state: RefCell::new(EngineState {
                m_last_tick_time_point: Instant::now(),
                m_frame_count: 0,
                m_average_duration: 0.0,
                m_fps: 0,
                m_is_editor_mode: false,
            }),
        }
    }

    pub fn resumed(&mut self, event_loop: &ActiveEventLoop, engine: Weak<RefCell<Engine>>) {
        self.m_runtime_context.resumed_instance(event_loop, engine);
    }
    pub fn initialize(&mut self){
        self.m_state.borrow_mut().m_last_tick_time_point = Instant::now();
    }

    pub fn shutdown_engine(&self){
        self.m_runtime_context.shutdown_systems();
    }

    pub fn calculate_delta_time(&self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.m_state.borrow().m_last_tick_time_point).as_secs_f32();
        self.m_state.borrow_mut().m_last_tick_time_point = now;
        delta_time
    }

    pub fn tick_one_frame(&self, delta_time: f32) -> Result<bool> {
        self.m_runtime_context
            .render_system()
            .borrow_mut()
            .swap_logic_render_data();
        self.logical_tick(delta_time);
        self.calculate_fps(delta_time);
        self.renderer_tick(delta_time)?;
        self.m_runtime_context
            .window_system()
            .borrow()
            .set_title(&format!("Editor - FPS: {}", self.m_state.borrow().m_fps));
        Ok(!self
            .m_runtime_context
            .window_system()
            .borrow()
            .should_close())
    }

    pub fn is_editor_mode(&self) -> bool {
        self.m_state.borrow().m_is_editor_mode
    }

    pub fn set_editor_mode(&self, value: bool) {
        self.m_state.borrow_mut().m_is_editor_mode = value;
    }
}

impl Engine {
    fn renderer_tick(&self, delta_time: f32) -> Result<()>{
        let window_size = self.m_runtime_context.window_system().borrow().get_window_size();
        self.m_runtime_context.render_system().borrow().update_engine_content_viewport(
            0.0, 0.0, window_size.0 as f32,  window_size.1 as f32
        );
        self.m_runtime_context.render_system().borrow().tick(
            &self.m_runtime_context.debugdraw_manager(),
            &self.m_runtime_context.ui_runtime(),
            &self.m_runtime_context.asset_manager(),
            &self.m_runtime_context.config_manager(),
            delta_time
        )?;
        Ok(())
    }

    fn logical_tick(&self, delta_time: f32) {
        let render_system = self.m_runtime_context.render_system().borrow();
        let rhi = render_system.get_rhi().borrow();
        let swapchain_info = rhi.get_swapchain_info();
        let viewport = [swapchain_info.extent.width as f32, swapchain_info.extent.height as f32];
        {
            let mut ui_runtime = self.m_runtime_context.ui_runtime().borrow_mut();
            ui_runtime.set_viewport(viewport);
            ui_runtime.new_frame();
        }

        self.m_runtime_context.world_manager().borrow_mut().tick(
            &self,
            &self.m_runtime_context.asset_manager(),
            &self.m_runtime_context.config_manager(),
            delta_time
        );
        self.m_runtime_context.input_system().borrow_mut().tick(
            &self.m_runtime_context.window_system().borrow(),
            &self.m_runtime_context.render_system().borrow(),
            &self.m_runtime_context.ui_runtime(),
            delta_time
        );
    }
    fn calculate_fps(&self, delta_time: f32) {
        self.m_state.borrow_mut().calculate_fps(delta_time);
    }
}

impl Engine {
    pub fn world_manager(&self) -> &RefCell<WorldManager> {
        &self.m_runtime_context.world_manager()
    }
    pub fn window_system(&self) -> &Rc<RefCell<WindowSystem>> {
        &self.m_runtime_context.window_system()
    }
    pub fn render_system(&self) -> &RefCell<RenderSystem> {
        &self.m_runtime_context.render_system()
    }
    pub fn ui_runtime(&self) -> &RefCell<UiRuntime> {
        &self.m_runtime_context.ui_runtime()
    }
    pub fn input_system(&self) -> &RefCell<InputSystem> {
        &self.m_runtime_context.input_system()
    }
    pub fn asset_manager(&self) -> &AssetManager {
        &self.m_runtime_context.asset_manager()
    }
    pub fn config_manager(&self) -> &ConfigManager {
        &self.m_runtime_context.config_manager()
    }
}

impl EngineState {
    fn calculate_fps(&mut self, delta_time: f32) {
        self.m_frame_count += 1;

        if self.m_frame_count == 1 {
            self.m_average_duration = delta_time;
        } else{
            self.m_average_duration = self.m_average_duration * (1.0 - S_FPS_ALPHA) + delta_time * S_FPS_ALPHA;
        }
        self.m_fps = (1.0 / self.m_average_duration) as u32;
    }
}