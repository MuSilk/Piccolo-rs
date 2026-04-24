use std::env;

use anyhow::anyhow;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::{
    engine::{Engine, System},
    function::{framework::scene::scene::SceneTrait, render::window_system::WindowCreateInfo},
};

pub struct App {
    engine: Engine,
}

impl App {
    pub fn new() -> Self {
        let _ = pretty_env_logger::try_init();
        let executable_path = env::current_exe().unwrap();
        let config_file_path = executable_path
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent directory"))
            .unwrap()
            .join("PiccoloEditor.ini");
        Self {
            engine: Engine::new(&config_file_path),
        }
    }

    pub fn set_window_create_info(&mut self, window_create_info: WindowCreateInfo) {
        self.engine.set_window_create_info(window_create_info);
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }

    pub fn add_system<T>(&mut self, system: T)
    where
        T: System + 'static,
    {
        self.engine.systems.borrow_mut().push(Box::new(system));
    }

    pub fn add_scene<T: SceneTrait + 'static>(&mut self, scene: T) {
        let mut world_manager = self.engine.world_manager().borrow_mut();
        world_manager.add_scene(scene);
    }

    pub fn set_default_scene(&mut self, scene_url: &str) {
        let mut world_manager = self.engine.world_manager().borrow_mut();
        world_manager.set_default_scene(scene_url);
    }

    pub fn register_on_key_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, &KeyEvent, bool),
    {
        let mut window_system = self.engine.window_system().borrow_mut();
        window_system.register_on_key_func(f);
    }

    pub fn register_on_mouse_button_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, ElementState, MouseButton),
    {
        let mut window_system = self.engine.window_system().borrow_mut();
        window_system.register_on_mouse_button_func(f);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.resumed(event_loop);
        Engine::initialize(&self.engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                let minimized = {
                    let window_system = self.engine.window_system().borrow();
                    window_system.is_minimized()
                };
                if !event_loop.exiting() && !minimized {
                    let delta_time = self.engine.calculate_delta_time();
                    if !self.engine.tick_one_frame(delta_time).unwrap() {
                        self.engine.shutdown_engine();
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Resized(size) => {
                let mut window_system = self.engine.window_system().borrow_mut();
                window_system.on_window_size(size);
            }
            WindowEvent::CloseRequested => {
                self.engine.shutdown_engine();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let window_system = self.engine.window_system().borrow();
                window_system.on_key(&self.engine, device_id, &event, is_synthetic);
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let mut window_system = self.engine.window_system().borrow_mut();
                window_system.on_mouse_button(&self.engine, device_id, state, button);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let window_system = self.engine.window_system().borrow();
                window_system.on_cursor_pos(&self.engine, device_id, position);
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let window_system = self.engine.window_system().borrow();
                window_system.on_mouse_wheel(&self.engine, device_id, delta, phase);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let window_system = self.engine.window_system().borrow();
                window_system.on_mouse_motion(&self.engine, device_id, delta);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window_system = self.engine.window_system().borrow();
        window_system.request_redraw();
    }
}
