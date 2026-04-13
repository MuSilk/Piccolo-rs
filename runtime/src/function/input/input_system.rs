use std::{cell::RefCell};

use bitflags::bitflags;
use winit::{dpi::PhysicalPosition, event::{DeviceId, ElementState, KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}};

use crate::{engine::Engine, function::{render::{render_system::RenderSystem, window_system::WindowSystem}, ui::ui2::{UiInputSnapshot, UiRuntime}}};


bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug)]
    pub struct GameCommand: u32 {
        const forward       = 1 << 0;
        const backward      = 1 << 1;
        const left          = 1 << 2;
        const right         = 1 << 3;
        const up            = 1 << 4;
        const down          = 1 << 5;
        const jump          = 1 << 6;
        const squat         = 1 << 7;
        const sprint        = 1 << 8;
        const fire          = 1 << 9;
        const free_camera   = 1 << 10;
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
    m_cursor_pos: [f32; 2],
    m_mouse_down: [bool; 3],
}

impl InputSystem {

    pub fn get_game_command(&self) -> &GameCommand {
        &self.m_game_command
    }

    pub fn is_mouse_button_down(&self, button: usize) -> bool {
        button < 3 && self.m_mouse_down[button]
    }

    pub fn reset_game_command(&mut self) {
        self.m_game_command = GameCommand::empty();
    }

    pub fn on_key(
        &mut self,
        engine: &Engine,
        device_id: DeviceId, 
        event: &KeyEvent, 
        is_synthetic: bool
    ) {
        if event.state == ElementState::Pressed {
            if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                engine.window_system().borrow().request_close();
                return;
            }
        }
        if !engine.is_editor_mode() {
            let window_system = engine.window_system().borrow();
            self.on_key_in_game_mode(&window_system, device_id, event, is_synthetic);
        }
    }

    fn on_key_in_game_mode(
        &mut self, 
        window_system: &WindowSystem,
        _device_id: DeviceId, 
        event: &KeyEvent, 
        _is_synthetic: bool
    ) { 
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
                            KeyCode::KeyQ => {
                                self.m_game_command |= GameCommand::up;
                            }
                            KeyCode::KeyE => {
                                self.m_game_command |= GameCommand::down;
                            }
                            KeyCode::ControlLeft => {
                                self.m_game_command |= GameCommand::squat;
                            }
                            KeyCode::AltLeft => {
                                let mode =  window_system.get_focus_mode();
                                window_system.set_focus_mode(!mode);
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
                            KeyCode::KeyQ => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::up;
                            } 
                            KeyCode::KeyE => {
                                self.m_game_command &= GameCommand::all() ^ GameCommand::down;
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

    pub fn on_mouse_motion(
        &mut self, 
        window_system: &WindowSystem,
        _device_id: DeviceId, 
        delta: (f64, f64)
    ) {
        if window_system.get_focus_mode() {
            self.m_cursor_delta_x = -delta.0 as i32;
            self.m_cursor_delta_y = -delta.1 as i32;
        }
    }

    pub fn on_cursor_pos(&mut self, _device_id: DeviceId, position: PhysicalPosition<f64>) {
        self.m_cursor_pos = [position.x as f32, position.y as f32];
    }

    pub fn on_mouse_button(&mut self, _device_id: DeviceId, state: ElementState, button: MouseButton) {
        let idx = match button {
            MouseButton::Left => Some(0),
            MouseButton::Right => Some(1),
            MouseButton::Middle => Some(2),
            _ => None,
        };
        if let Some(idx) = idx {
            self.m_mouse_down[idx] = state == ElementState::Pressed;
        }
    }

    fn clear(&mut self) {
        self.m_cursor_delta_x = 0;
        self.m_cursor_delta_y = 0;
    }
    
    fn calculate_cursor_delta_angles(
        &mut self,
        window_system: &WindowSystem,
        render_system: &RenderSystem,
    ) {
        let window_size = window_system.get_window_size();

        if window_size.0 < 1 || window_size.1 < 1 {
            return;
        }

        let render_camera = render_system.get_render_camera();
        let fov = render_camera.borrow().get_fov();

        let cursor_delta_x = (self.m_cursor_delta_x as f32 / window_size.0 as f32 * 4.0).to_radians();
        let cursor_delta_y = (self.m_cursor_delta_y as f32 / window_size.1 as f32 * 4.0).to_radians();

        self.m_cursor_delta_yaw = cursor_delta_x * fov.x;
        self.m_cursor_delta_pitch = -cursor_delta_y * fov.y;

    }

    pub fn tick(
        &mut self, 
        window_system: &WindowSystem,
        render_system: &RenderSystem,
        ui_runtime: &RefCell<UiRuntime>,
        _delta_time: f32
    ) {
        ui_runtime.borrow_mut().update_input(UiInputSnapshot {
            mouse_pos: self.m_cursor_pos,
            mouse_down: self.m_mouse_down,
            mouse_wheel: 0.0,
        });
        self.calculate_cursor_delta_angles(window_system, render_system);
        self.clear();        

        if window_system.get_focus_mode() {
            self.m_game_command &= GameCommand::all() ^ GameCommand::invalid;
        }
        else {
            self.m_game_command |= GameCommand::invalid;
        }
    }
}