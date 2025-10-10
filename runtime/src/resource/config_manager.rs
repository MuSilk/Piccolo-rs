use std::{fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};


#[derive(Default)]
pub struct ConfigManager {
    m_root_folder: PathBuf,
    m_asset_folder: PathBuf,
    m_schema_folder: PathBuf,
    m_editor_big_icon_path: PathBuf,
    m_editor_small_icon_path: PathBuf,
    m_editor_font_path: PathBuf,
    m_jolt_physics_asset_folder: PathBuf,

    m_default_world_url: String,
    m_global_rendering_res_url: String,
    m_global_particle_res_url: String,
}

impl ConfigManager {
    pub fn initialize(&mut self, config_file_path: &Path) {
        let file = File::open(config_file_path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let seperate_pos = line.find('=');
            match seperate_pos {
                Some(pos) => {
                    let key = &line[0..pos];
                    let value = &line[pos+1..line.len()];
                    match key {
                        "BinaryRootFolder" => {
                            self.m_root_folder = config_file_path.parent().unwrap().join(value);
                        }
                        "AssetFolder" => {
                            self.m_asset_folder = self.m_root_folder.join(value);
                        }
                        "SchemaFolder" => {
                            self.m_schema_folder = self.m_root_folder.join(value);
                        }
                        "DefaultWorld" => {
                            self.m_default_world_url = value.to_string();
                        }
                        "BigIconFile" => {
                            self.m_editor_big_icon_path = self.m_root_folder.join(value);
                        }
                        "SmallIconFile" => {
                            self.m_editor_small_icon_path = self.m_root_folder.join(value);
                        }
                        "FontFile" => {
                            self.m_editor_font_path = self.m_root_folder.join(value);
                        }
                        "GlobalRenderingRes" => {
                            self.m_global_rendering_res_url = value.to_string();
                        }
                        "GlobalParticleRes" => {
                            self.m_global_particle_res_url = value.to_string();
                        }
                        "JoltAssetFolder" => {
                            self.m_jolt_physics_asset_folder = self.m_root_folder.join(value);
                        }
                        _ => {}
                    }
                }
                None => {}
            }
        }
    }

    pub fn get_root_folder(&self) -> &Path {
        &self.m_root_folder
    }

    pub fn get_asset_folder(&self) -> &Path {
        &self.m_asset_folder
    }

    pub fn get_schema_folder(&self) -> &Path {
        &self.m_schema_folder
    }

    pub fn get_editor_big_icon_path(&self) -> &Path {
        &self.m_editor_big_icon_path
    }

    pub fn get_editor_small_icon_path(&self) -> &Path {
        &self.m_editor_small_icon_path
    }

    pub fn get_editor_font_path(&self) -> &Path {
        &self.m_editor_font_path
    }

    pub fn get_default_world_url(&self) -> &str {
        &self.m_default_world_url
    }

    pub fn get_global_rendering_res_url(&self) -> &str {
        &self.m_global_rendering_res_url
    }

    pub fn get_global_particle_res_url(&self) -> &str {
        &self.m_global_particle_res_url
    }

    pub fn get_jolt_physics_asset_folder(&self) -> &Path {
        &self.m_jolt_physics_asset_folder
    }
}