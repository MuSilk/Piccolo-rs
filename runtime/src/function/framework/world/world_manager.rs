use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use crate::function::framework::level::level::{Level, LevelExt};

#[derive(Default)]
pub struct WorldManager {
    m_is_world_loaded: bool,
    m_current_world_url: String,

    m_loaded_levels: HashMap<String, Rc<RefCell<Level>>>,
    m_current_level: Weak<RefCell<Level>>,
}

impl WorldManager {
    pub fn initialize(&mut self, m_current_world_url: &str) {
        self.m_is_world_loaded = false;
        self.m_current_world_url = m_current_world_url.to_string();
    }

    pub fn tick(&mut self, delta_time: f32) {
        if !self.m_is_world_loaded {
            self.load_world();
        }
        if let Some(level) = self.m_current_level.upgrade() {
            let level = level.borrow();
            level.tick(delta_time);
        }
    }

    pub fn load_world(&mut self) {
        let level = Level::new();
        level.load();
        self.m_current_level = Rc::downgrade(&level);
        self.m_loaded_levels.insert("Default".to_string(), level);
        self.m_is_world_loaded = true;
    }
}