extern crate self as runtime;

pub mod app;
pub mod core;
pub mod engine;
pub mod function;
pub mod resource;
mod shader;
pub use runtime_derive::ComponentTrait as ComponentTrait;
pub use winit::event as event;
pub use winit::keyboard as keyboard;