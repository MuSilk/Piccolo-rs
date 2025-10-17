
use reflection::{reflection::ReflectionPtr};

use crate::function::framework::component::component::{ComponentTrait};

#[derive(Default)]
pub struct ObjectInstanceRes {
    pub m_name: String,
    pub m_definition: String,

    pub m_instanced_components: Vec<ReflectionPtr<dyn ComponentTrait>>, 
}