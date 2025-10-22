use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use anyhow::Result;
use log::info;
use crate::{function::{framework::level::level::{Level, LevelExt}, global::global_context::RuntimeGlobalContext}, resource::{config_manager, res_type::common::world::WorldRes}};

#[derive(Default)]
pub struct WorldManager {
    m_is_world_loaded: bool,
    m_current_world_url: String,
    m_current_world_resource: WorldRes,

    m_loaded_levels: HashMap<String, Rc<RefCell<Level>>>,
    m_current_level: Weak<RefCell<Level>>,
}

impl WorldManager {
    pub fn initialize(&mut self) {
        self.m_is_world_loaded = false;
        let config_manager = RuntimeGlobalContext::get_config_manager().borrow();
        self.m_current_world_url = config_manager.get_default_world_url().to_string();
    }

    pub fn tick(&mut self, delta_time: f32) {
        if !self.m_is_world_loaded {
            self.load_world().unwrap();
        }
        if let Some(level) = self.m_current_level.upgrade() {
            let mut level = level.borrow_mut();
            level.tick(delta_time);
        }
    }

    fn load_world(&mut self) -> Result<()> {
        info!("Loading world: {}", self.m_current_world_url);
        let assert_manager = RuntimeGlobalContext::get_asset_manager().borrow();
        let world_res: WorldRes = assert_manager.load_asset(&self.m_current_world_url)?;

        self.load_level(&world_res.m_default_level_url.as_str())?;

        self.m_current_world_resource = world_res;
        self.m_is_world_loaded = true;
        info!("World load succeed!");
        Ok(())
    }

    fn load_level(&mut self, level_url: &str) -> Result<()> {
        let mut level = Level::new();
        level.load(level_url);
        self.m_current_level = Rc::downgrade(&level);
        self.m_loaded_levels.insert(level_url.to_string(), level);
        Ok(())
    }
}