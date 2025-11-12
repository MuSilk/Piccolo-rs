use std::{cell::RefCell, collections::HashMap, rc::{Rc}};

use anyhow::Result;
use log::info;
use crate::{function::{framework::scene::scene::{SceneTrait}, global::global_context::RuntimeGlobalContext}, resource::res_type::common::world::WorldRes};

#[derive(Default)]
pub struct WorldManager {
    m_is_world_loaded: bool,
    m_current_world_url: String,
    m_current_world_resource: WorldRes,

    m_scenes: HashMap<String, Rc<RefCell<dyn SceneTrait>>>,
    m_current_scene: Option<Rc<RefCell<dyn SceneTrait>>>,
}

impl WorldManager {
    pub fn initialize(&mut self) {
        self.m_is_world_loaded = false;
        let config_manager = RuntimeGlobalContext::get_config_manager().borrow();
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

    pub fn tick(&mut self, delta_time: f32) {
        if !self.m_is_world_loaded {
            self.load_world().unwrap();
        }
        if let Some(scene) = self.m_current_scene.as_ref() {
            let mut scene = scene.borrow_mut();
            if !scene.is_loaded() {
                scene.load();
            }
            scene.tick(delta_time);
        }
    }

    fn load_world(&mut self) -> Result<()> {
        info!("Loading world: {}", self.m_current_world_url);
        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        let world_res: WorldRes = assert_manager.load_asset(&self.m_current_world_url)?;
        self.m_current_world_resource = world_res;
        self.m_is_world_loaded = true;
        info!("World load succeed!");
        Ok(())
    }
}