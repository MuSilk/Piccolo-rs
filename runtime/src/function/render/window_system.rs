use std::{cell::Cell, collections::HashMap, rc::Rc};

use anyhow::Result;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceId, ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window, WindowLevel},
};

use crate::engine::Engine;

#[derive(Clone)]
pub struct WindowCreateInfo {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub is_fullscreen: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub ime_allowed: bool,
}

impl Default for WindowCreateInfo {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "Example Window",
            is_fullscreen: false,
            transparent: false,
            decorations: true,
            always_on_top: false,
            ime_allowed: false,
        }
    }
}

type OnKeyFunc = dyn Fn(&Engine, DeviceId, &KeyEvent, bool);
type OnMouseButtonFunc = dyn Fn(&Engine, DeviceId, ElementState, MouseButton);
type OnCursorPosFunc = dyn Fn(&Engine, DeviceId, PhysicalPosition<f64>);
type OnMouseWheelFunc = dyn Fn(&Engine, DeviceId, MouseScrollDelta, TouchPhase);
type OnImeFunc = dyn Fn(&Engine, &Ime);
type OnWindowSizeFunc = dyn Fn(&Engine, PhysicalSize<u32>);
type OnMouseMotionFunc = dyn Fn(&Engine, DeviceId, (f64, f64));

#[derive(Default)]
pub struct WindowSystem {
    pub m_window: Option<Rc<Window>>,
    m_width: u32,
    m_height: u32,
    m_is_focus_mode: Cell<bool>,
    m_should_close: Cell<bool>,
    m_minimized: bool,

    m_on_key_func: Vec<Box<OnKeyFunc>>,
    m_on_mouse_button_func: Vec<Box<OnMouseButtonFunc>>,
    m_on_cursor_pos_func: Vec<Box<OnCursorPosFunc>>,
    m_on_mouse_wheel_func: Vec<Box<OnMouseWheelFunc>>,
    m_on_ime_func: Vec<Box<OnImeFunc>>,
    m_on_mouse_motion_func: Vec<Box<OnMouseMotionFunc>>,

    m_is_mouse_button_down: HashMap<MouseButton, bool>,
}

impl WindowSystem {
    pub fn initialize(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_create_info: WindowCreateInfo,
    ) -> Result<()> {
        self.m_width = window_create_info.width;
        self.m_height = window_create_info.height;

        let attr = Window::default_attributes()
            .with_title(window_create_info.title)
            .with_inner_size(PhysicalSize::new(self.m_width, self.m_height))
            .with_transparent(window_create_info.transparent)
            .with_decorations(window_create_info.decorations)
            .with_window_level(if window_create_info.always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });

        let window = event_loop.create_window(attr)?;
        window.set_ime_allowed(window_create_info.ime_allowed);
        if window_create_info.ime_allowed {
            window.set_ime_cursor_area(
                PhysicalPosition::new(16.0, 16.0),
                PhysicalSize::new(self.m_width.max(1), self.m_height.max(1)),
            );
        }
        self.m_window = Some(Rc::new(window));
        Ok(())
    }

    pub fn set_title(&self, title: &str) {
        self.m_window.as_ref().unwrap().set_title(title);
    }

    pub fn get_window(&self) -> &Rc<Window> {
        &self.m_window.as_ref().unwrap()
    }

    pub fn is_minimized(&self) -> bool {
        self.m_minimized
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        let physical_size = self.m_window.as_ref().unwrap().inner_size();
        (physical_size.width, physical_size.height)
    }

    pub fn request_redraw(&self) {
        self.m_window.as_ref().unwrap().request_redraw();
    }

    pub fn request_close(&self) {
        self.m_should_close.set(true);
    }

    pub fn should_close(&self) -> bool {
        self.m_should_close.get()
    }

    pub fn register_on_key_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, &KeyEvent, bool),
    {
        self.m_on_key_func.push(Box::new(f));
    }

    pub fn register_on_mouse_button_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, ElementState, MouseButton),
    {
        self.m_on_mouse_button_func.push(Box::new(f));
    }

    pub fn register_on_cursor_pos_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, PhysicalPosition<f64>),
    {
        self.m_on_cursor_pos_func.push(Box::new(f));
    }

    pub fn register_on_mouse_wheel_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, MouseScrollDelta, TouchPhase),
    {
        self.m_on_mouse_wheel_func.push(Box::new(f));
    }

    pub fn register_on_ime_func<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, &Ime),
    {
        self.m_on_ime_func.push(Box::new(f));
    }

    pub fn register_on_mouse_motion<F>(&mut self, f: F)
    where
        F: 'static + Fn(&Engine, DeviceId, (f64, f64)),
    {
        self.m_on_mouse_motion_func.push(Box::new(f));
    }

    pub fn on_window_size(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            self.m_minimized = true;
        } else {
            self.m_minimized = false;
        }
    }

    pub fn on_key(
        &self,
        engine: &Engine,
        device_id: DeviceId,
        event: &KeyEvent,
        is_synthetic: bool,
    ) {
        self.m_on_key_func
            .iter()
            .for_each(|f| f(engine, device_id, event, is_synthetic));
    }

    pub fn on_mouse_button(
        &mut self,
        engine: &Engine,
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    ) {
        self.m_is_mouse_button_down
            .insert(button, state == ElementState::Pressed);
        self.m_on_mouse_button_func
            .iter()
            .for_each(|f| f(engine, device_id, state, button));
    }

    pub fn on_mouse_wheel(
        &self,
        engine: &Engine,
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    ) {
        self.m_on_mouse_wheel_func
            .iter()
            .for_each(|f| f(engine, device_id, delta, phase));
    }

    pub fn on_ime(&self, engine: &Engine, ime: Ime) {
        self.m_on_ime_func
            .iter()
            .for_each(|f| f(engine, &ime));
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self
            .m_is_mouse_button_down
            .get(&button)
            .unwrap_or_else(|| &false)
    }

    pub fn on_cursor_pos(
        &self,
        engine: &Engine,
        device_id: DeviceId,
        physical_position: PhysicalPosition<f64>,
    ) {
        self.m_on_cursor_pos_func
            .iter()
            .for_each(|f| f(engine, device_id, physical_position));
    }

    pub fn on_mouse_motion(&self, engine: &Engine, device_id: DeviceId, mouse_motion: (f64, f64)) {
        self.m_on_mouse_motion_func
            .iter()
            .for_each(|f| f(engine, device_id, mouse_motion));
    }

    pub fn get_focus_mode(&self) -> bool {
        self.m_is_focus_mode.get()
    }

    pub fn set_focus_mode(&self, mode: bool) {
        self.m_is_focus_mode.set(mode);
        if mode {
            self.m_window
                .as_ref()
                .unwrap()
                .set_cursor_grab(CursorGrabMode::Locked)
                .unwrap();
            self.m_window.as_ref().unwrap().set_cursor_visible(false);
        } else {
            self.m_window
                .as_ref()
                .unwrap()
                .set_cursor_grab(CursorGrabMode::None)
                .unwrap();
            self.m_window.as_ref().unwrap().set_cursor_visible(true);
        }
    }

    pub fn drag_window(&self) -> Result<()> {
        self.m_window.as_ref().unwrap().drag_window()?;
        Ok(())
    }
}
