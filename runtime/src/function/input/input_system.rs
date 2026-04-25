use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, Ime, KeyEvent, MouseButton},
};

use crate::engine::Engine;

pub trait InputSystem {
    fn on_key(
        &mut self,
        _engine: &Engine,
        _device_id: DeviceId,
        _event: &KeyEvent,
        _is_synthetic: bool,
    ) {
    }
    fn on_mouse_motion(&mut self, _engine: &Engine, _device_id: DeviceId, _delta: (f64, f64)) {}
    fn on_cursor_pos(&mut self, _device_id: DeviceId, _position: PhysicalPosition<f64>) {}
    fn on_mouse_button(
        &mut self,
        _device_id: DeviceId,
        _state: ElementState,
        _button: MouseButton,
    ) {
    }
    fn on_ime(&mut self, _ime: &Ime) {}
    fn tick(&mut self, _engine: &Engine, _delta_time: f32) {}
}
