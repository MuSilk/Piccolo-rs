use std::{path::Path, time::Instant};

use crate::runtime::function::global::global_context::RuntimeGlobalContext;

pub struct Engine {
    tick_time_point_last : Instant,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            tick_time_point_last: Instant::now(),
        }
    }
}

impl Engine {
    pub fn initialize(&self, config_file_path: &Path){
        RuntimeGlobalContext::start_systems(config_file_path);
    }

    pub fn shutdown(&self){
        
    }

    pub fn calculate_delta_time(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.tick_time_point_last).as_secs_f32();
        self.tick_time_point_last = now;
        delta_time
    }

    pub fn tick_one_frame(&mut self, delta_time: f32) -> bool {
        self.renderer_tick(delta_time);
        let ctx = RuntimeGlobalContext::global().borrow();
        return !ctx.m_window_system.borrow().should_close();
    }
}

impl Engine {
    fn renderer_tick(&mut self, delta_time: f32){
        let ctx = RuntimeGlobalContext::global().borrow();
        let mut render_system = ctx.m_render_system.borrow_mut();
        render_system.tick(delta_time);
    }
}