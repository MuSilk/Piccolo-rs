use std::{path::Path, time::Instant};

use anyhow::Result;
use winit::event_loop::ActiveEventLoop;

use crate::{function::global::global_context::RuntimeGlobalContext};

pub static mut G_IS_EDITOR_MODE: bool = false;

pub struct Engine {
    m_is_quit: bool,
    m_last_tick_time_point: Instant,
    m_average_duration: f32,
    m_frame_count: u32,
    m_fps: u32,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            m_is_quit: false,
            m_last_tick_time_point: Instant::now(),
            m_average_duration: 0.0,
            m_frame_count: 0,
            m_fps: 0,
        }
    }
}

impl Engine {

    pub fn start_engine(&self, event_loop: &ActiveEventLoop, config_file_path: &Path) -> Result<()> {
        RuntimeGlobalContext::start_systems(event_loop, config_file_path)?;
        Ok(())  
    }
    pub fn initialize(&mut self){
        self.m_last_tick_time_point = Instant::now();
    }

    pub fn shutdown_engine(&self){
        RuntimeGlobalContext::global().shutdown_systems();
    }

    pub fn calculate_delta_time(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.m_last_tick_time_point).as_secs_f32();
        self.m_last_tick_time_point = now;
        delta_time
    }

    pub fn tick_one_frame(&mut self, delta_time: f32) -> Result<bool> {
        RuntimeGlobalContext::get_render_system().borrow_mut().swap_logic_render_data();
        Self::logical_tick(delta_time);
        Self::renderer_tick(delta_time)?;
        Ok(!RuntimeGlobalContext::get_window_system().borrow().should_close())
    }
}

impl Engine {
    fn renderer_tick(delta_time: f32) -> Result<()>{
        RuntimeGlobalContext::get_render_system().borrow_mut().tick(delta_time)?;
        Ok(())
    }

    fn logical_tick(delta_time: f32) {
        let ctx = RuntimeGlobalContext::global();
        ctx.m_world_manager.borrow_mut().tick(delta_time);
    }

    const S_FPS_ALPHA: f32 = 1.0 / 100.0;
    fn calculate_fps(&mut self, delta_time: f32) {
        self.m_frame_count += 1;

        if self.m_frame_count == 1 {
            self.m_average_duration = delta_time;
        } else{
            self.m_average_duration = self.m_average_duration * (1.0 - Self::S_FPS_ALPHA) + delta_time * Self::S_FPS_ALPHA;
        }
        self.m_fps = (1.0 / self.m_average_duration) as u32;
    }
}