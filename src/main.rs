use anyhow::{Result};
use winit::event_loop::EventLoop;

mod application;
mod vulkan;
mod utils;
mod surface;
use crate::application::App;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
