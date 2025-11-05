use serde::{Deserialize, Serialize};

use crate::function::framework::{resource::resource::Resource};

#[derive(Serialize, Deserialize, Default)]
pub struct ObjectInstanceRes {
    pub m_name: String,
    pub m_definition: String,
    pub m_instanced_components: Vec<Box<dyn Resource>>,
}