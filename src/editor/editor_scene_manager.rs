use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::runtime::function::render::render_camera::RenderCamera;

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
}