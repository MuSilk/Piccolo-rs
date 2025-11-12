use std::{cell::RefCell, rc::Rc};

use runtime::{core::math::{transform::Transform, vector3::Vector3}, engine::Engine, function::{framework::{component::{camera_component::CameraComponent, character_component::CharacterComponent, component::ComponentTrait, motor_component::MotorComponent, transform_component::TransformComponent}, resource::component::motor::MotorComponentRes, scene::scene::SceneTrait}, global::global_context::RuntimeGlobalContext}};

use crate::{ecs::controller::CharacterController, world::World};


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

        let world = Rc::new(RefCell::new(World::new_box(&mut self.scene)));
        self.scene.add_resource(world.clone());

        let object = self.scene.spawn();
        let character = Box::new(CharacterComponent::new());
        let camera = Box::new(CameraComponent::new());
        let mut transform = Box::new(TransformComponent::default());
        let mut trans = Transform::default();
        trans.set_position(Vector3::new(64.0, 64.0, 256.0));
        transform.post_load_resource(object, trans);
        let controller = Box::new(CharacterController::new(world));
        let mut motor = Box::new(MotorComponent::new(controller));
        let motor_res: MotorComponentRes = RuntimeGlobalContext::get_asset_manager().borrow()
            .load_asset("asset/objects/character/components/player.motor.json").unwrap();
        motor.post_load_resources(&motor_res);
        let components = vec![
            RefCell::new(character as Box<dyn ComponentTrait>),
            RefCell::new(camera as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
            RefCell::new(motor as Box<dyn ComponentTrait>),
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

            self.scene.query_triple_mut::<CharacterComponent, TransformComponent, MotorComponent>()
            .for_each(|(mut character, mut transform, mut motor)|
            {
                motor.tick(delta_time, &mut transform);

                if character.m_rotation_dirty {
                    transform.set_rotation(character.m_rotation_buffer);
                    character.m_rotation_dirty = false;
                }

                if motor.get_is_moving() {
                    character.m_rotation_buffer = character.m_rotation;
                    transform.set_rotation(character.m_rotation_buffer);
                    character.m_rotation_dirty = true;
                }

                let new_position = motor.get_target_position();
                character.m_position = *new_position;
            });
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