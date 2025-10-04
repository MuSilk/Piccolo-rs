use std::{cell::RefCell, sync::Mutex};

use crate::runtime::function::render::debugdraw::debug_draw_group::DebugDrawGroup;


#[derive(Default)]
pub struct DebugDrawContext {
    pub m_debug_draw_groups: Vec<RefCell<DebugDrawGroup>>,
    m_mutex: Mutex<()>,
}

impl DebugDrawContext {
    pub fn try_get_or_create_debug_draw_group(&mut self, name: &str) -> &RefCell<DebugDrawGroup> {
        let _guard = self.m_mutex.lock();

        if self.m_debug_draw_groups.iter().find(|g| g.borrow().m_name == name).is_none() {
            let mut debug_draw_group = DebugDrawGroup::default();
            debug_draw_group.initialize();
            debug_draw_group.set_name(name);
            self.m_debug_draw_groups.push(RefCell::new(debug_draw_group));
        }
        return &self.m_debug_draw_groups.iter().find(|g| g.borrow().m_name == name).unwrap();
    }

    pub fn clear(&mut self){
        let _guard = self.m_mutex.lock();
        for g in self.m_debug_draw_groups.iter_mut() {
            g.borrow_mut().clear();
        }
        self.m_debug_draw_groups.clear();
    }

    pub fn tick(&mut self, delta_time: f32){
        self.remove_dead_primitives(delta_time);
    }
}

impl DebugDrawContext {
    fn remove_dead_primitives(&mut self, delta_time: f32){
        let _guard = self.m_mutex.lock();
        for g in self.m_debug_draw_groups.iter_mut() {
            g.borrow_mut().remove_dead_primitives(delta_time);
        }

    }
}