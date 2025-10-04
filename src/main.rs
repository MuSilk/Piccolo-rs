use std::env;

use anyhow::{anyhow, Result};
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowId};


use crate::runtime::engine::Engine;
use crate::runtime::function::global::global_context::RuntimeGlobalContext;

pub mod runtime;
pub mod shader;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = WinitApp::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[derive(Default)]
struct WinitApp{
    minimized: bool,
    engine: Engine,
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let executable_path = env::current_exe().unwrap();
        let config_file_path = executable_path.parent().ok_or_else(||
            anyhow!("Failed to get parent directory")
        ).unwrap();
        self.engine.initialize();
        RuntimeGlobalContext::start_systems(event_loop, config_file_path).unwrap();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested 
                if !event_loop.exiting() &&!self.minimized => {
                    let delta_time = self.engine.calculate_delta_time();
                    if !self.engine.tick_one_frame(delta_time).unwrap() {
                        event_loop.exit();
                    }  
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.minimized = true;
                } else {
                    self.minimized = false;
                }
            }
            WindowEvent::CloseRequested => {
                RuntimeGlobalContext::global().borrow_mut().shutdown_systems();
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        RuntimeGlobalContext::global().borrow().m_window_system.borrow().request_redraw();
    }

}