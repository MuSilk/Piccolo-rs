pub mod axis_aligned;
mod math;
pub mod matrix3;
pub mod matrix4;
pub mod quaternion;
pub mod transform;
pub mod vector2;
pub mod vector3;
pub mod vector4;

pub use math::look_at as look_at;
pub use math::perspective as perspective;
pub use math::orthographic_projection_01 as orthographic_projection_01;