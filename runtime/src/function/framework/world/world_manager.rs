use std::{cell::RefCell, collections::HashMap, rc::{Rc}};

use anyhow::Result;
use log::info;
use crate::{engine::Engine, function::framework::scene::scene::SceneTrait, resource::{asset_manager::AssetManager, config_manager::ConfigManager, res_type::common::world::WorldRes}};

#[derive(Default)]
pub struct WorldManager {
    m_is_world_loaded: bool,
    m_current_world_url: String,
    m_current_world_resource: WorldRes,

    m_scenes: HashMap<String, Rc<RefCell<dyn SceneTrait>>>,
    m_current_scene: Option<Rc<RefCell<dyn SceneTrait>>>,
}

impl WorldManager {
    pub fn initialize(
        &mut self,
        config_manager: &ConfigManager,
    ) {
        self.m_is_world_loaded = false;
        self.m_current_world_url = config_manager.get_default_world_url().to_string();
    }

    pub fn add_scene<T: SceneTrait + 'static>(&mut self, scene: T) {
        self.m_scenes.insert(scene.get_url(), Rc::new(RefCell::new(scene)));
    }

    pub fn set_default_scene(&mut self, scene_name: &str) {
        self.m_current_scene = self.m_scenes.get(scene_name).map(|s| s.clone());
    }

    pub fn get_current_scene(&self) -> &Option<Rc<RefCell<dyn SceneTrait>>> {
        &self.m_current_scene
    }

    pub fn tick(
        &mut self, 
        engine: &Engine,
        asset_manager: &AssetManager,
        config_manager: &ConfigManager,
        delta_time: f32
    ) {
        if !self.m_is_world_loaded {
            self.load_world(asset_manager, config_manager).unwrap();
        }
        if let Some(scene) = self.m_current_scene.as_ref() {
            let mut scene = scene.borrow_mut();
            if !scene.is_loaded() {
                scene.load(engine);
            }
            scene.tick(engine, delta_time);
        }
    }

    fn load_world(
        &mut self,
        asset_manager: &AssetManager,
        config_manager: &ConfigManager,
    ) -> Result<()> {
        info!("Loading world: {}", self.m_current_world_url);
        let world_res: WorldRes = asset_manager.load_asset(config_manager, &self.m_current_world_url)?;
        self.m_current_world_resource = world_res;
        self.m_is_world_loaded = true;
        info!("World load succeed!");
        Ok(())
    }
}