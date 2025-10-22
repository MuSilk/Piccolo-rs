use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Default)]
pub struct WorldRes {
    #[serde(rename = "name")] 
    pub m_name: String,
    #[serde(rename = "level_urls")] 
    pub m_level_urls: Vec<String>,
    #[serde(rename = "default_level_url")] 
    pub m_default_level_url: String,
}