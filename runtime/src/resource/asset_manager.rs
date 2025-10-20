use std::path::PathBuf;

use crate::{function::global::global_context::RuntimeGlobalContext};


#[derive(Default)]
pub struct AssetManager {}

impl AssetManager {
    pub fn get_full_path(&self, relative_path: &str) -> PathBuf {
        let global = RuntimeGlobalContext::global();
        let config_manager = global.m_config_manager.borrow();
        let root_folder = config_manager.get_root_folder();
        root_folder.join(relative_path)
    }
}