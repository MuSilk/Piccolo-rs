use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use log::error;
use serde::de::DeserializeOwned;

use crate::resource::config_manager::ConfigManager;


pub struct AssetManager {}

impl AssetManager {

    pub fn new() -> Self {
        AssetManager {
        }
    }

    pub fn get_full_path(
        &self, 
        config_manager: &ConfigManager,
        relative_path: &str,
    ) -> PathBuf {
        let root_folder = config_manager.get_root_folder();
        root_folder.join(relative_path)
    }

    pub fn load_asset<AssetType : DeserializeOwned>(
        &self, 
        config_manager: &ConfigManager,
        asset_url: &str
    ) -> Result<AssetType> {
        let asset_path = self.get_full_path(config_manager, asset_url);
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

    pub fn save_asset<AssetType : serde::Serialize>(
        &self, asset_url: &str, 
        config_manager: &ConfigManager,
        asset: AssetType
    ) -> Result<()> {
        let asset_path = self.get_full_path(config_manager, asset_url);
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