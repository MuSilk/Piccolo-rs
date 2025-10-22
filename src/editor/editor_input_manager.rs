use std::{cell::RefCell, rc::{Rc}};

use bitflags::bitflags;
use runtime::{core::math::{vector2::Vector2, vector3::Vector3}, engine::G_IS_EDITOR_MODE, function::global::global_context::RuntimeGlobalContext};
use winit::{dpi::PhysicalPosition, event::{DeviceId, ElementState, KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}};

use crate::{editor::{editor_global_context::EditorGlobalContext}};

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct EditorCommand: u32 {
        const camera_left       = 1 << 0;
        const camera_back       = 1 << 1;
        const camera_forward    = 1 << 2;
        const camera_right      = 1 << 3;
        const camera_up         = 1 << 4;
        const camera_down       = 1 << 5;
        const translation_mode  = 1 << 6;
        const rotation_mode     = 1 << 7;
        const scale_mode        = 1 << 8;
        const exit              = 1 << 9;
        const delete_object     = 1 << 10;
    }
}

#[derive(Default)]
pub struct EditorInputManager {
    m_engine_window_pos: Vector2,
    m_engine_window_size: Vector2,
    m_mouse_x: f32,
    m_mouse_y: f32,
    m_camera_speed: f32,

    m_cursor_on_axis: usize,
    m_editor_command: EditorCommand,
}

impl EditorInputManager {
    fn on_cursor_pos(&mut self, _device_id: DeviceId, position: PhysicalPosition<f64>) {
        if !unsafe{G_IS_EDITOR_MODE} {
            return;
        }
        let angular_velocity = 180.0 / (self.m_engine_window_size.x).max(self.m_engine_window_size.y);
        if self.m_mouse_x >= 0.0 && self.m_mouse_y >=0.0 {
            let editor_global = EditorGlobalContext::global().borrow();
            let window_system = editor_global.m_window_system.upgrade().unwrap();
            if window_system.borrow().is_mouse_button_down(MouseButton::Right) {
                let scene_manager = editor_global.m_scene_manager.borrow();
                let camera = scene_manager.get_editor_camera();
                let camera = camera.upgrade().unwrap();
                camera.borrow_mut().rotate_camera(&(Vector2::new(
                    position.y as f32 - self.m_mouse_y,
                    position.x as f32 - self.m_mouse_x
                ) * angular_velocity));
            }
            else if window_system.borrow().is_mouse_button_down(MouseButton::Left) {

            }
            else{

            }
        }
        self.m_mouse_x = position.x as f32;
        self.m_mouse_y = position.y as f32;
    }

    fn on_key(&mut self, device_id: DeviceId, event: &KeyEvent, is_synthetic: bool) { 
        if unsafe{G_IS_EDITOR_MODE} {
            self.on_key_editor_mode(device_id, event, is_synthetic);
        }
    }

    fn on_key_editor_mode(&mut self, _device_id: DeviceId, event: &KeyEvent, _is_synthetic: bool) {
        if event.state == ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(code) => {
                    match code {
                        KeyCode::KeyA => {
                            self.m_editor_command |= EditorCommand::camera_left;
                        }
                        KeyCode::KeyS => {
                            self.m_editor_command |= EditorCommand::camera_back;
                        }
                        KeyCode::KeyW => {
                            self.m_editor_command |= EditorCommand::camera_forward;
                        } 
                        KeyCode::KeyD => {
                            self.m_editor_command |= EditorCommand::camera_right;
                        }
                        KeyCode::KeyQ => {
                            self.m_editor_command |= EditorCommand::camera_up;
                        }
                        KeyCode::KeyE => {
                            self.m_editor_command |= EditorCommand::camera_down;
                        }
                        KeyCode::KeyT => {
                            self.m_editor_command |= EditorCommand::translation_mode;
                        }
                        KeyCode::KeyR => {
                            self.m_editor_command |= EditorCommand::rotation_mode;
                        }
                        KeyCode::KeyY => {
                            self.m_editor_command |= EditorCommand::scale_mode;
                        }
                        KeyCode::Delete => {
                            self.m_editor_command |= EditorCommand::delete_object;
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        else if event.state == ElementState::Released {
            match event.physical_key {
                PhysicalKey::Code(code) => {
                    match code {
                        KeyCode::KeyA => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_left;
                        }
                        KeyCode::KeyS => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_back;
                        }
                        KeyCode::KeyW => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_forward;
                        } 
                        KeyCode::KeyD => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_right;
                        }
                        KeyCode::KeyQ => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_up;
                        }
                        KeyCode::KeyE => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::camera_down;
                        }
                        KeyCode::KeyT => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::translation_mode;
                        }
                        KeyCode::KeyR => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::rotation_mode;
                        }
                        KeyCode::KeyY => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::scale_mode;
                        }
                        KeyCode::Delete => {
                            self.m_editor_command &= EditorCommand::all() ^ EditorCommand::delete_object;
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}

#[derive(Default)]
pub struct  WrappedEditorInputManager(Rc<RefCell<EditorInputManager>>);

impl WrappedEditorInputManager {
    pub fn initialize(&self) {
        self.0.borrow_mut().m_engine_window_size = Vector2::new(1024.0, 768.0);
        self.0.borrow_mut().m_camera_speed = 0.05;
        self.register_input();
    }

    pub fn tick(&self, _delta_time: f32) {
        self.process_editor_command();
    }
}

impl WrappedEditorInputManager {
    fn register_input(&self) {
        let mut window_system = RuntimeGlobalContext::get_window_system().borrow_mut();
        let this = Rc::downgrade(&self.0);
        window_system.register_on_cursor_pos_func(move |device_id, position| {
            let this = this.upgrade().unwrap();
            this.borrow_mut().on_cursor_pos(device_id, position);
        });
        let this = Rc::downgrade(&self.0);
        window_system.register_on_key_func(move |device_id, event, is_synthetic| {
            let this = this.upgrade().unwrap();
            this.borrow_mut().on_key(device_id, event, is_synthetic);
        });
    }

    fn process_editor_command(&self) {
        let camera_speed = self.0.borrow().m_camera_speed;
        let editor_global = EditorGlobalContext::global().borrow();
        let scene_manager = editor_global.m_scene_manager.borrow();
        let camera = scene_manager.get_editor_camera();
        let camera = camera.upgrade().unwrap();
        let mut editor_camera =  camera.borrow_mut();
        let camera_rotate = editor_camera.rotation().conjugate();
        let mut camera_relative_pos = Vector3::new(0.0, 0.0, 0.0);
        
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_forward) {
            camera_relative_pos += camera_rotate * Vector3::new(0.0, camera_speed, 0.0); 
        }
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_back) {
            camera_relative_pos += camera_rotate * Vector3::new(0.0, -camera_speed, 0.0);
        }
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_left) {
            camera_relative_pos += camera_rotate * Vector3::new(-camera_speed, 0.0, 0.0);
        }
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_right) {
            camera_relative_pos += camera_rotate * Vector3::new(camera_speed, 0.0, 0.0);
        }
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_up) {
            camera_relative_pos += camera_rotate * Vector3::new(0.0, 0.0, camera_speed);
        }
        if self.0.borrow().m_editor_command.contains(EditorCommand::camera_down) {
            camera_relative_pos += camera_rotate * Vector3::new(0.0, 0.0, -camera_speed);
        }
        editor_camera.move_camera(&camera_relative_pos);
    }
}