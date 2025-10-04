use std::rc::{Rc, Weak};

use anyhow::Result;
use winit::{dpi::LogicalSize, event_loop::{ActiveEventLoop}, window::Window};

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

#[derive(Default)]
pub struct WindowSystem{
    pub window: Option<Rc<Window>>,
    width: u32,
    height: u32,
}

impl WindowSystem {
    pub fn initialize(&mut self, event_loop: &ActiveEventLoop, window_create_info: WindowCreateInfo) -> Result<()> {
        self.width = window_create_info.width;
        self.height = window_create_info.height;

        let attr = Window::default_attributes()
            .with_title(window_create_info.title)
            .with_inner_size(LogicalSize::new(self.width,self.height));

        let window = event_loop.create_window(attr)?;
        self.window = Some(Rc::new(window));
        Ok(())
    }

    pub fn set_title(&self, title: &str) {
        self.window.as_ref().unwrap().set_title(title);
    }   

    pub fn get_window(&self) -> Weak<Window> {
        Rc::downgrade(&self.window.as_ref().unwrap())
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        let physical_size = self.window.as_ref().unwrap().inner_size();
        (physical_size.width, physical_size.height)
    }
    pub fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    pub fn should_close(&self) -> bool {
        false
    }
}