use std::{collections::HashMap, rc::Rc};

use anyhow::Result;
use winit::{dpi::{LogicalSize, PhysicalPosition}, event::{DeviceId, ElementState, KeyEvent, MouseButton}, event_loop::ActiveEventLoop, window::Window};

pub struct WindowCreateInfo{
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub is_fullscreen: bool,
}

impl Default for WindowCreateInfo{
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "Example Window",
            is_fullscreen: false,
        }
    }
}

type OnKeyFunc = dyn Fn(DeviceId, &KeyEvent, bool);
type OnMouseButtonFunc = dyn Fn(DeviceId, ElementState, MouseButton);
type OnCursorPosFunc = dyn Fn(DeviceId, PhysicalPosition<f64>);

#[derive(Default)]
pub struct WindowSystem{
    pub m_window: Option<Rc<Window>>,
    m_width: u32,
    m_height: u32,
    m_is_focus_mode: bool,

    m_on_key_func: Vec<Box<OnKeyFunc>>,
    m_on_mouse_button_func: Vec<Box<OnMouseButtonFunc>>,
    m_on_cursor_pos_func: Vec<Box<OnCursorPosFunc>>,

    m_is_mouse_button_down: HashMap<MouseButton, bool>,
}

impl WindowSystem {
    pub fn initialize(&mut self, event_loop: &ActiveEventLoop, window_create_info: WindowCreateInfo) -> Result<()> {
        self.m_width = window_create_info.width;
        self.m_height = window_create_info.height;

        let attr = Window::default_attributes()
            .with_title(window_create_info.title)
            .with_inner_size(LogicalSize::new(self.m_width,self.m_height));

        let window = event_loop.create_window(attr)?;
        self.m_window = Some(Rc::new(window));
        Ok(())
    }

    pub fn set_title(&self, title: &str) {
        self.m_window.as_ref().unwrap().set_title(title);
    }   

    pub fn get_window(&self) -> &Rc<Window> {
        &self.m_window.as_ref().unwrap()
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        let physical_size = self.m_window.as_ref().unwrap().inner_size();
        (physical_size.width, physical_size.height)
    }
    pub fn request_redraw(&self) {
        self.m_window.as_ref().unwrap().request_redraw();
    }

    pub fn should_close(&self) -> bool {
        false
    }

    pub fn register_on_key_func<F>(&mut self, f: F) 
    where
        F: 'static + Fn(DeviceId, &KeyEvent, bool),
    {
        self.m_on_key_func.push(Box::new(f));
    }

    pub fn register_on_mouse_button_func<F>(&mut self, f: F) 
    where
        F: 'static + Fn(DeviceId, ElementState, MouseButton),
    {
        self.m_on_mouse_button_func.push(Box::new(f));
    }

    pub fn register_on_cursor_pos_func<F>(&mut self, f: F) 
    where
        F: 'static + Fn(DeviceId, PhysicalPosition<f64>),
    {
        self.m_on_cursor_pos_func.push(Box::new(f));
    }

    pub fn on_key(&self, device_id: DeviceId, event: &KeyEvent, is_synthetic: bool) {
        self.m_on_key_func.iter().for_each(|f| f(device_id, event, is_synthetic));
    }

    pub fn on_mouse_button(&mut self, device_id: DeviceId, state: ElementState, button: MouseButton) {
        self.m_is_mouse_button_down.insert(button, state == ElementState::Pressed);
        self.m_on_mouse_button_func.iter().for_each(|f| f(device_id, state, button));
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.m_is_mouse_button_down.get(&button).unwrap_or_else(|| &false)
    }

    pub fn on_cursor_pos(&self, device_id: DeviceId, physical_position: PhysicalPosition<f64>) {
        self.m_on_cursor_pos_func.iter().for_each(|f| f(device_id, physical_position));
    }

}