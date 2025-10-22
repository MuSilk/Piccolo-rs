use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use log::error;
use serde::de::DeserializeOwned;

use crate::{function::global::global_context::RuntimeGlobalContext};


#[derive(Default)]
pub struct AssetManager {}

impl AssetManager {
    pub fn get_full_path(&self, relative_path: &str) -> PathBuf {
        let config_manager = RuntimeGlobalContext::get_config_manager().borrow();
        let root_folder = config_manager.get_root_folder();
        root_folder.join(relative_path)
    }

    pub fn load_asset<AssetType : DeserializeOwned>(&self, asset_url: &str) -> Result<AssetType> {
        let asset_path = self.get_full_path(asset_url);
        let reader = std::fs::File::open(asset_path).map(std::io::BufReader::new);
        if let Err(e) = reader {
            error!("Failed to open asset file {}: {}", asset_url, e);
            return Err(anyhow!("Failed to open asset file {}: {}", asset_url, e));
        }
        let asset_json = serde_json::from_reader(reader.unwrap());
        if let Err(e) = asset_json {
            error!("Failed to parse asset file {}: {}", asset_url, e);
            return Err(anyhow!("Failed to parse asset file {}: {}", asset_url, e));
        }
        Ok(asset_json.unwrap())
    }

    pub fn save_asset<AssetType : serde::Serialize>(&self, asset_url: &str, asset: AssetType) -> Result<()> {
        let asset_path = self.get_full_path(asset_url);
        let writer = std::fs::File::create(asset_path).map(std::io::BufWriter::new);
        if let Err(e) = writer {
            error!("Failed to create asset file {}: {}", asset_url, e);
            return Err(anyhow!("Failed to create asset file {}: {}", asset_url, e));
        }
        let result = serde_json::to_writer_pretty(writer.unwrap(), &asset);
        if let Err(e) = result {
            error!("Failed to write asset file {}: {}", asset_url, e);
            return Err(anyhow!("Failed to write asset file {}: {}", asset_url, e));
        }
        Ok(())
    }
}