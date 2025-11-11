use std::{cell::RefCell, rc::Rc};

use bitflags::bitflags;
use winit::{event::{DeviceId, ElementState, KeyEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::{engine::Engine, function::{global::global_context::RuntimeGlobalContext}};


bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct GameCommand: u32 {
        const forward       = 1 << 0;
        const backward      = 1 << 1;
        const left          = 1 << 2;
        const right         = 1 << 3;
        const jump          = 1 << 4;
        const squat         = 1 << 5;
        const sprint        = 1 << 6;
        const fire          = 1 << 7;
        const free_camera   = 1 << 8;
        const invalid       = 1 << 31;
    }
}

#[derive(Default)]
pub struct InputSystem {
    m_cursor_delta_x : i32,
    m_cursor_delta_y : i32,

    pub m_cursor_delta_yaw: f32,
    pub m_cursor_delta_pitch: f32,

    m_game_command: GameCommand,
}

impl InputSystem {

    pub fn get_game_command(&self) -> &GameCommand {
        &self.m_game_command
    }

    pub fn reset_game_command(&mut self) {
        self.m_game_command = GameCommand::empty();
    }

    fn on_key(&mut self, device_id: DeviceId, event: &KeyEvent, is_synthetic: bool) {
        if !Engine::is_editor_mode() {
            self.on_key_in_game_mode(device_id, event, is_synthetic);
        }
    }

    fn on_key_in_game_mode(&mut self, _device_id: DeviceId, event: &KeyEvent, _is_synthetic: bool) { 
        self.m_game_command &= GameCommand::all() ^ GameCommand::jump;

        match event.state {
            ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::KeyA => {
                                self.m_game_command |= GameCommand::left;
                            }
                            KeyCode::KeyS => {
                                self.m_game_command |= GameCommand::backward;
                            }
                            KeyCode::KeyW => {
                                self.m_game_command |= GameCommand::forward;
                            } 
                            KeyCode::KeyD => {
                                self.m_game_command |= GameCommand::right;
                            }
                            KeyCode::Space => {
                                self.m_game_command |= GameCommand::jump;
                            }
                            KeyCode::ControlLeft => {
                                self.m_game_command |= GameCommand::squat;
                            }
                            KeyCode::AltLeft => {
                                let mode =  RuntimeGlobalContext::get_window_system().borrow().get_focus_mode();
                                RuntimeGlobalContext::get_window_system().borrow().set_focus_mode(!mode);
                            }
                            KeyCode::ShiftLeft => {
                                self.m_game_command |= GameCommand::sprint;
                            }
                            KeyCode::KeyF => {
                                self.m_game_command ^= GameCommand::free_camera;
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            ElementState::Released => {
                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::KeyA => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::left;
                            }
                            KeyCode::KeyS => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::backward;
                            }
                            KeyCode::KeyW => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::forward;
                            } 
                            KeyCode::KeyD => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::right;
                            }
                            KeyCode::ControlLeft => {
                                self.m_game_command &= GameCommand::squat;
                            }
                            KeyCode::ShiftLeft => {
                                self.m_game_command &= GameCommand::sprint;
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn on_mouse_motion(&mut self, _device_id: DeviceId, delta: (f64, f64)) {
        if RuntimeGlobalContext::get_window_system().borrow().get_focus_mode() {
            self.m_cursor_delta_x = -delta.0 as i32;
            self.m_cursor_delta_y = -delta.1 as i32;
        }
    }

    fn clear(&mut self) {
        self.m_cursor_delta_x = 0;
        self.m_cursor_delta_y = 0;
    }
    
    fn calculate_cursor_delta_angles(&mut self) {
        let window_size = RuntimeGlobalContext::get_window_system().borrow().get_window_size();

        if window_size.0 < 1 || window_size.1 < 1 {
            return;
        }

        let render_system = RuntimeGlobalContext::get_render_system().borrow();
        let render_camera = render_system.get_render_camera();
        let fov = render_camera.borrow().get_fov();

        let cursor_delta_x = (self.m_cursor_delta_x as f32 / window_size.0 as f32 * 4.0).to_radians();
        let cursor_delta_y = (self.m_cursor_delta_y as f32 / window_size.1 as f32 * 4.0).to_radians();

        self.m_cursor_delta_yaw = cursor_delta_x * fov.x;
        self.m_cursor_delta_pitch = -cursor_delta_y * fov.y;

    }

    pub fn tick(&mut self, _delta_time: f32) {
        self.calculate_cursor_delta_angles();
        self.clear();        

        if RuntimeGlobalContext::get_window_system().borrow().get_focus_mode() {
            self.m_game_command &= GameCommand::all() ^ GameCommand::invalid;
        }
        else {
            self.m_game_command |= GameCommand::invalid;
        }
    }
}

pub trait InputSystemExt {
    fn initialize(&self);
}

impl InputSystemExt for Rc<RefCell<InputSystem>> {
    fn initialize(&self) {
        let mut window_system = RuntimeGlobalContext::get_window_system().borrow_mut();
        let this = Rc::downgrade(&self);
        window_system.register_on_key_func(move |device_id, event, is_synthetic| {
            let this = this.upgrade().unwrap();
            this.borrow_mut().on_key(device_id, event, is_synthetic);
        });
        let this = Rc::downgrade(&self);
        window_system.register_on_mouse_motion(move |device_id, position| {
            let this = this.upgrade().unwrap();
            this.borrow_mut().on_mouse_motion(device_id, position);
        });
    }
}