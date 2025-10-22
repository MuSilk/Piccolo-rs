use serde::{Deserialize, Serialize};

use crate::resource::res_type::common::object::ObjectInstanceRes;


#[derive(Serialize, Deserialize, Default)]
pub struct LevelRes {
    #[serde(rename = "objects")] 
    pub m_objects: Vec<ObjectInstanceRes>,
}