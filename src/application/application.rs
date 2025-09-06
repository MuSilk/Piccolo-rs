use anyhow::{Result};
use winit::{application::ApplicationHandler, 
    dpi::LogicalSize, event::WindowEvent, 
    event_loop::ActiveEventLoop, 
    window::{Window, WindowId}
};
use log::*;

use crate::{ vulkan::{ VulkanContext}};

pub struct App {
    window: Option<Window>,
    vulkan_context: Option<VulkanContext>,
    minimized: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Window::default_attributes()
            .with_title("Test Window")
            .with_inner_size(LogicalSize::new(1024, 768));

        let window = event_loop.create_window(window).unwrap();
        self.vulkan_context = Some(VulkanContext::create(&window).unwrap());
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested if !event_loop.exiting() &&!self.minimized => {
                self.render().unwrap_or_else(|e|{error!("{}",e)});
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.minimized = true;
                } else {
                    self.minimized = false;
                    if let Some(vulkan_context) = self.vulkan_context.as_mut(){
                        vulkan_context.resized = true;
                    }
                }
            }
            WindowEvent::CloseRequested => {
                self.vulkan_context.as_mut().unwrap().destroy();
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

}

impl App {

    pub fn new() -> Self {
        Self {
            window: None,
            vulkan_context: None,
            minimized: false,
        }
    }

    fn render(&mut self)->Result<()>{
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return Ok(())
        };

        let ctx = match self.vulkan_context.as_mut() {
            Some(ctx) => ctx,
            None => return Ok(())
        };
        
        ctx.render(window)?;

        Ok(())
    }
}