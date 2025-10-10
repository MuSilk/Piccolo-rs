use reflection::reflection_derive::ReflectFields;



#[derive(ReflectFields)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}