use std::{cell::RefCell, env, rc::Rc};

use anyhow::anyhow;
use winit::{application::ApplicationHandler, event::{Event, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::WindowId};

use crate::{engine::Engine, function::{framework::{scene::scene::SceneTrait}, global::global_context::RuntimeGlobalContext}};

pub struct App{
    engine: Rc<RefCell<Engine>>,
    systems: Vec<Box<dyn System>>,
}

pub trait System {
    fn initialize(&mut self, _engine: &Rc<RefCell<Engine>>) {}

    fn tick(&mut self, _delta_time: f32) {}
}

impl App {

    pub fn new() -> Self {
        let _ = pretty_env_logger::try_init();
        let executable_path = env::current_exe().unwrap();
        let config_file_path = executable_path.parent().ok_or_else(||
            anyhow!("Failed to get parent directory")
        ).unwrap().join("PiccoloEditor.ini");
        Self { 
            engine: Rc::new(RefCell::new(Engine::new(&config_file_path))), 
            systems: Default::default() 
        }
    }
    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }

    pub fn add_system<T>(&mut self, system: T) where T: System + 'static {
        self.systems.push(Box::new(system));
    }

    pub fn add_scene<T: SceneTrait + 'static>(&mut self, scene: T) {
        let mut world_manager = RuntimeGlobalContext::get_world_manager().borrow_mut();
        world_manager.add_scene(scene);
    }

    pub fn set_default_scene(&mut self, scene_url: &str) {
        let mut world_manager = RuntimeGlobalContext::get_world_manager().borrow_mut();
        world_manager.set_default_scene(scene_url);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.borrow().resumed(event_loop);
        self.engine.borrow_mut().initialize();
        self.systems.iter_mut().for_each(|s| s.initialize(&self.engine));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        {
            let render_system = RuntimeGlobalContext::get_render_system().borrow();
            render_system.handle_event(&Event::<()>::WindowEvent{
                window_id,
                event: event.clone(),
            });
        }

        match event {
            WindowEvent::RedrawRequested => {
                let minimized = {
                    RuntimeGlobalContext::get_window_system().borrow().is_minimized()
                };
                if !event_loop.exiting() &&!minimized {
                    let delta_time = self.engine.borrow_mut().calculate_delta_time();
                    self.systems.iter_mut().for_each(|s|s.tick(delta_time));
                    if !self.engine.borrow_mut().tick_one_frame(delta_time).unwrap() {
                        event_loop.exit();
                    }  
                }
            }
            WindowEvent::Resized(size) => {
                let mut window_system = RuntimeGlobalContext::get_window_system().borrow_mut();
                window_system.on_window_size(size);
            }
            WindowEvent::CloseRequested => {
                self.engine.borrow().shutdown_engine();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                let window_system = RuntimeGlobalContext::get_window_system().borrow();
                window_system.on_key(device_id, &event, is_synthetic);
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                let mut window_system = RuntimeGlobalContext::get_window_system().borrow_mut();
                window_system.on_mouse_button(device_id, state, button);
            }
            WindowEvent::CursorMoved { device_id, position } => {
                let window_system = RuntimeGlobalContext::get_window_system().borrow();
                window_system.on_cursor_pos(device_id, position);
            }
            WindowEvent::MouseWheel { device_id, delta, phase } => {
                let window_system = RuntimeGlobalContext::get_window_system().borrow();
                window_system.on_mouse_wheel(device_id, delta, phase);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        RuntimeGlobalContext::get_window_system().borrow().request_redraw();
    }

}