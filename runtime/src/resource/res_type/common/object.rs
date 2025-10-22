use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ObjectInstanceRes {
    pub m_name: String,
    pub m_definition: String,
}