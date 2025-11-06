use std::{cell::RefCell, rc::{Rc, Weak}};

use runtime::{core::math::vector2::Vector2, function::render::{render_camera::RenderCamera}};

use crate::editor_global_context::EditorGlobalContext;


#[derive(Default)]
pub struct EditorSceneManager {
    m_camera: Weak<RefCell<RenderCamera>>,
}

impl EditorSceneManager {
    pub fn initialize(&mut self) {

    }

    pub fn set_editor_camera(&mut self, camera: &Rc<RefCell<RenderCamera>>) {
        self.m_camera = Rc::downgrade(camera);
    }

    pub fn get_editor_camera(&self) -> &Weak<RefCell<RenderCamera>> {
        &self.m_camera
    }

    pub fn get_guid_of_picked_mesh(&self, picked_uv: &Vector2) -> u32 {
        let global = EditorGlobalContext::global();
        let render_system = global.borrow().m_render_system.upgrade().unwrap();
        render_system.borrow().get_guid_of_picked_mesh(picked_uv)
    }
}