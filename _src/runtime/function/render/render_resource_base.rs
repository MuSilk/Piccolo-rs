use std::any::Any;

pub struct RenderResourceBase{

}

pub trait RenderResourceBaseTrait: Any {
    
}

impl dyn RenderResourceBaseTrait {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}