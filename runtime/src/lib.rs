extern crate self as runtime;

pub mod app;
pub mod core;
pub mod engine;
pub mod function;
mod resource;
mod shader;
pub use runtime_derive::ComponentTrait as ComponentTrait;