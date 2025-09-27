use std::{cell::RefCell, rc::{Rc, Weak}, sync::Arc};

use anyhow::Result;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::WindowId};

use crate::{editor::editor_ui::EditorUI, runtime::{engine::Engine, function::{global::global_context::RuntimeGlobalContext, render::{render_system::RenderSystemInitInfo, window_system::WindowCreateInfo}, ui::window_ui::{WindowUI, WindowUIInitInfo}}}};

#[derive(Default)]
pub struct Editor{
    editor_ui: Rc<EditorUI>,
    engine_runtime: Weak<RefCell<Engine>>,
    event_loop: Option<EventLoop<()>>,
}

impl Editor {
    pub fn initialize(&mut self, engine: &Rc<RefCell<Engine>>) -> Result<()> {
        self.engine_runtime = Rc::downgrade(&engine);
        let global = RuntimeGlobalContext::global();
        let ui_init_info = WindowUIInitInfo{
            window_system: &global.m_window_system.lock().unwrap(),
            render_system: &global.m_render_system.lock().unwrap(),
        };
        self.editor_ui = Rc::new(EditorUI::default());
        self.editor_ui.as_ref().initialize(ui_init_info);

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        self.event_loop = Some(event_loop);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()>{
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run_app(self)?;
        Ok(())
    }

    pub fn shutdown(&mut self){}
}

impl ApplicationHandler for Editor {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let global = RuntimeGlobalContext::global();
        let window_create_info = WindowCreateInfo::default();
        let _ = global.m_window_system.lock().unwrap().initialize(event_loop, window_create_info);
        let render_init_info = RenderSystemInitInfo{
            window_system: &global.m_window_system.lock().unwrap(),
        };
        let _ = RuntimeGlobalContext::global().m_render_system.lock().unwrap().initialize(render_init_info);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested if !event_loop.exiting() => {
                let engine = self.engine_runtime.upgrade().unwrap();
                let mut engine = engine.borrow_mut();
                let delta_time = engine.calculate_delta_time();
                if !engine.tick_one_frame(delta_time) {
                    event_loop.exit();
                }   
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }


    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        RuntimeGlobalContext::global().m_window_system.lock().unwrap().request_redraw();
    }
}