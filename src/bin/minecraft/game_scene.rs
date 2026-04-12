//! `minecraft-ai` 独立场景：不引用 `crate::minecraft` 下任何模块。

use std::{cell::RefCell, rc::Rc};

use runtime::{
    core::math::transform::Transform,
    engine::Engine,
    function::{
        framework::{
            component::{
                camera_component::CameraComponent, character_component::CharacterComponent,
                component::ComponentTrait, transform_component::TransformComponent,
            },
            resource::component::motor::MotorComponentRes,
            scene::scene::SceneTrait,
        },
    },
};

use crate::{
    minecraft_motor_component::MinecraftMotorComponent, player_controller::AiPlayerController,
    voxel_world::VoxelWorld,
};

pub struct GameScene {
    pub inner: runtime::function::framework::scene::scene::Scene,
}

impl GameScene {
    pub fn new() -> Self {
        let mut inner = runtime::function::framework::scene::scene::Scene::new();
        inner.set_url("MinecraftAI");
        Self { inner }
    }
}

impl SceneTrait for GameScene {
    fn load(&mut self, engine: &Engine) {
        engine.window_system().borrow().set_focus_mode(true);

        let spawn = VoxelWorld::suggested_spawn();
        let world = Rc::new(RefCell::new(VoxelWorld::new_box(engine, &mut self.inner)));
        self.inner.add_resource(world.clone());

        let object = self.inner.spawn();
        let character = Box::new(CharacterComponent::new());
        let camera = Box::new(CameraComponent::new());
        let mut transform = Box::new(TransformComponent::default());
        let mut trans = Transform::default();
        trans.set_position(spawn);
        transform.post_load_resource(trans);
        let controller = Box::new(AiPlayerController::new(world));
        let mut motor = Box::new(MinecraftMotorComponent::new(controller));
        let motor_res: MotorComponentRes = engine
            .asset_manager()
            .load_asset(
                engine.config_manager(),
                "asset/minecraft-ai/player.motor.json",
            )
            .expect("player motor");
        motor.post_load_resources(&motor_res);
        let components = vec![
            RefCell::new(character as Box<dyn ComponentTrait>),
            RefCell::new(camera as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
            RefCell::new(motor as Box<dyn ComponentTrait>),
        ];
        self.inner.create_object(object, components);
        self.inner.set_loaded(true);
    }

    fn save(&self) {}

    fn tick(&mut self, engine: &Engine, delta_time: f32) {
        if !self.is_loaded() {
            return;
        }
        self.inner.tick_transform_components(engine);
        if !engine.is_editor_mode() {
            self.inner
                .query_triple_mut::<CharacterComponent, TransformComponent, MinecraftMotorComponent>()
                .for_each(|(mut character, mut transform, mut motor)| {
                    let input = engine.input_system().borrow();
                    motor.tick(&input, delta_time, &mut transform, character.get_rotation());
                    if character.m_rotation_dirty {
                        transform.set_rotation(character.m_rotation_buffer);
                        character.m_rotation_dirty = false;
                    }
                    if motor.get_is_moving() {
                        character.m_rotation_buffer = character.m_rotation;
                        transform.set_rotation(character.m_rotation_buffer);
                        character.m_rotation_dirty = true;
                    }
                    character.m_position = *motor.get_target_position();
                });
            let player_pos = self
                .inner
                .query_mut::<CharacterComponent>()
                .next()
                .map(|c| c.m_position);
            if let (Some(world_rc), Some(pos)) = (
                self.inner.get_mut_resource::<Rc<RefCell<Box<VoxelWorld>>>>(),
                player_pos,
            ) {
                world_rc.borrow_mut().update_streaming(&pos);
            }
            let input = engine.input_system().borrow();
            let render = engine.render_system().borrow();
            self.inner
                .tick_camera_components(&input, &render, delta_time);
        }
        let render = engine.render_system().borrow();
        self.inner.tick_mesh_components(&render);
    }

    fn get_url(&self) -> String {
        self.inner.get_url().clone()
    }

    fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }
}
