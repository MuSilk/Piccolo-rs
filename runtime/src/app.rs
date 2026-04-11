use std::{cell::RefCell, env, rc::Rc};

use anyhow::anyhow;
use winit::{application::ApplicationHandler, event::{DeviceEvent, DeviceId, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::WindowId};

use crate::{engine::{Engine, System}, function::framework::scene::scene::SceneTrait};

pub struct App{
    engine: Rc<RefCell<Engine>>,
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
        }
    }
    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }

    pub fn add_system<T>(&mut self, system: T) where T: System + 'static {
        self.engine.borrow_mut().systems.borrow_mut().push(Box::new(system));
    }

    pub fn add_scene<T: SceneTrait + 'static>(&mut self, scene: T) {
        let engine = self.engine.borrow();
        let mut world_manager = engine
            .world_manager()
            .borrow_mut();
        world_manager.add_scene(scene);
    }

    pub fn set_default_scene(&mut self, scene_url: &str) {
        let engine = self.engine.borrow();
        let mut world_manager = engine
            .world_manager()
            .borrow_mut();
        world_manager.set_default_scene(scene_url);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let engine_weak = Rc::downgrade(&self.engine);
        self.engine.borrow_mut().resumed(event_loop, engine_weak);
        Engine::initialize(&self.engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let minimized = {
                    let engine = self.engine.borrow();
                    let window_system = engine
                        .window_system()
                        .borrow();
                    window_system.is_minimized()
                };
                if !event_loop.exiting() &&!minimized {
                    let delta_time = self.engine.borrow().calculate_delta_time();
                    if !self.engine.borrow().tick_one_frame(delta_time).unwrap() {
                        event_loop.exit();
                    }  
                }
            }
            WindowEvent::Resized(size) => {
                let engine = self.engine.borrow();
                let mut window_system = engine
                    .window_system()
                    .borrow_mut();
                window_system.on_window_size(size);
            }
            WindowEvent::CloseRequested => {
                self.engine.borrow().shutdown_engine();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                let engine = self.engine.borrow();
                let window_system = engine
                    .window_system()
                    .borrow();
                window_system.on_key(device_id, &event, is_synthetic);
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                let engine = self.engine.borrow();
                let mut window_system = engine
                    .window_system()
                    .borrow_mut();
                window_system.on_mouse_button(device_id, state, button);
            }
            WindowEvent::CursorMoved { device_id, position } => {
                let engine = self.engine.borrow();
                let window_system = engine
                    .window_system()
                    .borrow();
                window_system.on_cursor_pos(device_id, position);
            }
            WindowEvent::MouseWheel { device_id, delta, phase } => {
                let engine = self.engine.borrow();
                let window_system = engine
                    .window_system()
                    .borrow();
                window_system.on_mouse_wheel(device_id, delta, phase);
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion  { delta } => {
                let engine = self.engine.borrow();
                let window_system = engine
                    .window_system()
                    .borrow();
                window_system.on_mouse_motion(device_id, delta);
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let engine = self.engine.borrow();
        let window_system = engine
            .window_system()
            .borrow();
        window_system.request_redraw();
    }

}