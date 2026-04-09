use std::{cell::RefCell, rc::{Rc, Weak}};

use runtime::{core::math::vector2::Vector2, engine::Engine, function::render::render_camera::RenderCamera};


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

    pub fn get_guid_of_picked_mesh(&self,engine: &Engine, picked_uv: &Vector2) -> u32 {
        let render_system = engine.render_system().borrow();
        render_system.get_guid_of_picked_mesh(picked_uv)
    }
}