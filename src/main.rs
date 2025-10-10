use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use runtime::engine::Engine;
use runtime::function::global::global_context::RuntimeGlobalContext;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowId};

pub mod editor;
use crate::editor::editor::Editor;
use crate::editor::editor_global_context::EditorGlobalContext;

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
    engine: Rc<RefCell<Engine>>,
    editor: Editor,
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let executable_path = env::current_exe().unwrap();
        let config_file_path = executable_path.parent().ok_or_else(||
            anyhow!("Failed to get parent directory")
        ).unwrap().join("PiccoloEditor.ini");
        self.engine.borrow().start_engine(event_loop, &config_file_path).unwrap();
        self.engine.borrow_mut().initialize();
        self.editor.initialize(&self.engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested 
                if !event_loop.exiting() &&!self.minimized => {
                    let delta_time = self.engine.borrow_mut().calculate_delta_time();
                    EditorGlobalContext::global().borrow().m_input_manager.tick(delta_time);
                    if !self.engine.borrow_mut().tick_one_frame(delta_time).unwrap() {
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
                self.engine.borrow().shutdown_engine();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                let global = RuntimeGlobalContext::global().borrow();
                let window_system = global.m_window_system.borrow();
                window_system.on_key(device_id, &event, is_synthetic);
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                let global = RuntimeGlobalContext::global().borrow();
                let mut window_system = global.m_window_system.borrow_mut();
                window_system.on_mouse_button(device_id, state, button);
            }
            WindowEvent::CursorMoved { device_id, position } => {
                let global = RuntimeGlobalContext::global().borrow();
                let window_system = global.m_window_system.borrow();
                window_system.on_cursor_pos(device_id, position);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        RuntimeGlobalContext::global().borrow().m_window_system.borrow().request_redraw();
    }

}