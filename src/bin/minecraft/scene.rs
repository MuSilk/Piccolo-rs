use std::cell::{RefCell};

use runtime::{engine::Engine, function::framework::{component::{camera_component::CameraComponent, character_component::CharacterComponent, component::ComponentTrait}, scene::scene::SceneTrait}};

use crate::world::World;


pub struct Scene {
    pub scene: runtime::function::framework::scene::scene::Scene,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = runtime::function::framework::scene::scene::Scene::new();
        scene.set_url("MineCraft");
        Self { scene }
    }
}

impl SceneTrait for Scene {
    fn load(&mut self){

        let object  = self.scene.spawn();
        let world = World::new_box(&mut self.scene);
        let components = vec![
            RefCell::new(world as Box<dyn ComponentTrait>),
        ];
        self.scene.create_object(object, components);

        let object = self.scene.spawn();
        let character = Box::new(CharacterComponent::new());
        let camera = Box::new(CameraComponent::new());
        let components = vec![
            RefCell::new(character as Box<dyn ComponentTrait>),
            RefCell::new(camera as Box<dyn ComponentTrait>),
        ];
        self.scene.create_object(object, components);

        self.scene.set_loaded(true);
    }

    fn save(&self) {
        
    }
    
    fn tick(&mut self, delta_time: f32) {
        if !self.is_loaded() {
            return;
        }
        self.scene.tick_transform_components(delta_time);
        self.scene.tick_mesh_components(delta_time);

        if !Engine::is_editor_mode() {
            self.scene.tick_camera_components(delta_time);
        }
    }

    fn get_url(&self) -> String {
        self.scene.get_url().clone()
    }

    fn is_loaded(&self) -> bool {
        self.scene.is_loaded()
    }
}