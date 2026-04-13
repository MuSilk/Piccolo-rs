//! `minecraft-ai` 独立场景：不引用 `crate::minecraft` 下任何模块。

use std::{cell::RefCell, path::Path, rc::Rc};
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use runtime::{
    core::math::{transform::Transform, vector3::Vector3},
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
    voxel_world::{VoxelKind, VoxelOverrideRecord, VoxelWorld},
};

/// 左键连续破坏间隔（秒）；松开后再按需立刻响应，故用同一数值做累计器初值。
const DIG_COOLDOWN: f32 = 0.22;
/// 右键连续放置间隔（秒）。
const PLACE_COOLDOWN: f32 = 0.28;
const SAVE_FILE_NAME: &str = "minecraft_ai.save.json";

#[derive(Serialize, Deserialize)]
struct GameSaveData {
    player_position: Vector3,
    overrides: Vec<VoxelOverrideRecord>,
}

fn save_file_path() -> PathBuf {
    PathBuf::from("save").join(SAVE_FILE_NAME)
}

fn slot_to_block(slot: u8) -> VoxelKind {
    match slot {
        1 => VoxelKind::Dirt,
        2 => VoxelKind::Stone,
        3 => VoxelKind::Sand,
        4 => VoxelKind::Plank,
        5 => VoxelKind::Brick,
        6 => VoxelKind::Log,
        7 => VoxelKind::Leaves,
        _ => VoxelKind::Dirt,
    }
}

/// 与 `AiPlayerController` 碰撞盒一致：脚底为 `feet` 的 AABB 是否与单位体素格相交。
fn player_aabb_overlaps_cell(feet: Vector3, wx: i32, wy: i32, wz: i32) -> bool {
    let min = feet;
    let max = Vector3::new(feet.x + 0.68, feet.y + 0.68, feet.z + 1.8);
    let bx0 = wx as f32;
    let by0 = wy as f32;
    let bz0 = wz as f32;
    min.x < bx0 + 1.0
        && max.x > bx0
        && min.y < by0 + 1.0
        && max.y > by0
        && min.z < bz0 + 1.0
        && max.z > bz0
}

pub struct GameScene {
    pub inner: runtime::function::framework::scene::scene::Scene,
    world: Option<Rc<RefCell<Box<VoxelWorld>>>>,
    hotbar_texture_id: Option<u32>,
    latest_player_position: Vector3,
    dig_repeat_accum: f32,
    place_repeat_accum: f32,
}

impl GameScene {
    pub fn new() -> Self {
        let mut inner = runtime::function::framework::scene::scene::Scene::new();
        inner.set_url("MinecraftAI");
        // 预置为间隔：首帧按下即可触发一次（否则需长按满一整段间隔才有反应）。
        Self {
            inner,
            world: None,
            hotbar_texture_id: None,
            latest_player_position: Vector3::ZERO,
            dig_repeat_accum: DIG_COOLDOWN,
            place_repeat_accum: PLACE_COOLDOWN,
        }
    }

    fn block_uv_rect(block: VoxelKind) -> [f32; 4] {
        let (tx, ty) = match block {
            VoxelKind::Dirt => (2, 0),
            VoxelKind::Stone => (3, 0),
            VoxelKind::Sand => (4, 0),
            VoxelKind::Plank => (5, 0),
            VoxelKind::Brick => (6, 0),
            VoxelKind::Log => (7, 0),
            VoxelKind::Leaves => (9, 0),
            _ => (2, 0),
        };
        const ATLAS: f32 = 16.0;
        let u0 = tx as f32 / ATLAS;
        let v0 = ty as f32 / ATLAS;
        let u1 = (tx + 1) as f32 / ATLAS;
        let v1 = (ty + 1) as f32 / ATLAS;
        [u0, v0, u1, v1]
    }

    fn draw_hotbar_ui(&mut self, engine: &Engine) {
        let selected_slot = {
            let input = engine.input_system().borrow();
            input.get_selected_block_slot().clamp(1, 7)
        };
        let slots: [VoxelKind; 7] = [
            VoxelKind::Dirt,
            VoxelKind::Stone,
            VoxelKind::Sand,
            VoxelKind::Plank,
            VoxelKind::Brick,
            VoxelKind::Log,
            VoxelKind::Leaves,
        ];

        let mut ui = engine.ui_runtime().borrow_mut();
        if self.hotbar_texture_id.is_none() {
            let tex_path = engine
                .asset_manager()
                .get_full_path(engine.config_manager(), "asset/minecraft-ai/texture/block.png");
            if let Ok(texture_id) = ui.load_texture_from_path(Path::new(&tex_path)) {
                self.hotbar_texture_id = Some(texture_id);
            }
            println!("hotbar_texture_id: {:?}", self.hotbar_texture_id);
        }
        let Some(texture_id) = self.hotbar_texture_id else {
            return;
        };

        let vp = ui.get_viewport();
        let slot_size = 42.0_f32;
        let pad = 8.0_f32;
        let total_w = slot_size * slots.len() as f32 + pad * (slots.len() as f32 - 1.0);
        let x0 = (vp[0] - total_w) * 0.5;
        let y0 = vp[1] - slot_size - 22.0;
        let clip = [0.0, 0.0, vp[0], vp[1]];

        ui.push_colored_rect(
            [x0 - 12.0, y0 - 10.0],
            [total_w + 24.0, slot_size + 20.0],
            [14, 16, 20, 160],
            clip,
        );

        for (i, block) in slots.iter().enumerate() {
            let x = x0 + i as f32 * (slot_size + pad);
            let selected = (i + 1) as u8 == selected_slot;
            let border_col = if selected {
                [255, 220, 128, 255]
            } else {
                [120, 128, 150, 220]
            };
            ui.push_colored_rect([x, y0], [slot_size, slot_size], [36, 40, 52, 220], clip);
            ui.push_colored_rect([x, y0], [slot_size, 2.0], border_col, clip);
            ui.push_colored_rect([x, y0 + slot_size - 2.0], [slot_size, 2.0], border_col, clip);
            ui.push_colored_rect([x, y0], [2.0, slot_size], border_col, clip);
            ui.push_colored_rect([x + slot_size - 2.0, y0], [2.0, slot_size], border_col, clip);
            ui.push_textured_rect(
                [x + 6.0, y0 + 6.0],
                [slot_size - 12.0, slot_size - 12.0],
                Self::block_uv_rect(*block),
                [255, 255, 255, 255],
                clip,
                texture_id,
            );
            ui.push_text_ascii(
                &(i + 1).to_string(),
                [x + 2.0, y0 + slot_size - 14.0],
                [7.0, 12.0],
                [230, 230, 235, 255],
                clip,
            );
        }
    }
}

impl SceneTrait for GameScene {
    fn load(&mut self, engine: &Engine) {
        engine.window_system().borrow().set_focus_mode(true);

        let mut spawn = VoxelWorld::suggested_spawn();
        let mut saved_overrides = Vec::new();
        if let Ok(text) = fs::read_to_string(save_file_path()) {
            if let Ok(save_data) = serde_json::from_str::<GameSaveData>(&text) {
                spawn = save_data.player_position;
                saved_overrides = save_data.overrides;
            }
        }
        let world = Rc::new(RefCell::new(VoxelWorld::new_box(engine, &mut self.inner)));
        self.inner.add_resource(world.clone());
        self.world = Some(world.clone());
        self.latest_player_position = spawn;
        if !saved_overrides.is_empty() {
            world.borrow_mut().replace_overrides(saved_overrides);
            world.borrow_mut().flush_voxel_mesh_sync(&spawn);
        }

        let object = self.inner.spawn();
        let mut character = Box::new(CharacterComponent::new());
        character.m_position = spawn;
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
        motor.align_spawn(spawn);
        let components = vec![
            RefCell::new(character as Box<dyn ComponentTrait>),
            RefCell::new(camera as Box<dyn ComponentTrait>),
            RefCell::new(transform as Box<dyn ComponentTrait>),
            RefCell::new(motor as Box<dyn ComponentTrait>),
        ];
        self.inner.create_object(object, components);
        self.inner.set_loaded(true);
    }

    fn save(&self) {
        let Some(world_rc) = self.world.as_ref() else {
            return;
        };
        let save = GameSaveData {
            player_position: self.latest_player_position,
            overrides: world_rc.borrow().snapshot_overrides(),
        };
        let Ok(text) = serde_json::to_string_pretty(&save) else {
            return;
        };
        let path = save_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, text);
    }
    

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
            if let Some(pos) = player_pos {
                self.latest_player_position = pos;
            }
            let input = engine.input_system().borrow();
            let render = engine.render_system().borrow();
            self.inner
                .tick_camera_components(&input, &render, delta_time);

            const REACH: f32 = 5.5;
            let (mouse_left, mouse_right) = {
                let inp = engine.input_system().borrow();
                (inp.is_mouse_button_down(0), inp.is_mouse_button_down(1))
            };
            let selected_block = {
                let inp = engine.input_system().borrow();
                slot_to_block(inp.get_selected_block_slot())
            };
            let cam_snap = self
                .inner
                .query_pair::<CameraComponent, CharacterComponent>()
                .next()
                .map(|(cam, ch)| (cam.m_position, cam.m_forward, ch.get_position()));

            let mut world_changed = false;
            if let (Some(world_rc), Some((origin, forward, feet))) = (self.world.as_ref(), cam_snap) {
                let mut world = world_rc.borrow_mut();
                if mouse_left {
                    self.dig_repeat_accum += delta_time;
                    while self.dig_repeat_accum >= DIG_COOLDOWN {
                        self.dig_repeat_accum -= DIG_COOLDOWN;
                        if let Some((x, y, z)) =
                            world.raycast_first_solid(origin, forward, REACH)
                        {
                            world.set_voxel(x, y, z, VoxelKind::Air);
                            world_changed = true;
                        }
                    }
                } else {
                    self.dig_repeat_accum = DIG_COOLDOWN;
                }
                if mouse_right {
                    self.place_repeat_accum += delta_time;
                    while self.place_repeat_accum >= PLACE_COOLDOWN {
                        self.place_repeat_accum -= PLACE_COOLDOWN;
                        if let Some((x, y, z)) =
                            world.raycast_place_cell(origin, forward, REACH)
                        {
                            if world.voxel_at(x, y, z) == VoxelKind::Air
                                && !player_aabb_overlaps_cell(feet, x, y, z)
                            {
                                world.set_voxel(x, y, z, selected_block);
                                world_changed = true;
                            }
                        }
                    }
                } else {
                    self.place_repeat_accum = PLACE_COOLDOWN;
                }
                world.flush_voxel_mesh_sync(&feet);
            }
            if world_changed {
                self.save();
            }

            if let (Some(world_rc), Some(pos)) = (self.world.as_ref(), player_pos) {
                world_rc
                    .borrow_mut()
                    .update_streaming(engine, &mut self.inner, &pos);
            }
            self.draw_hotbar_ui(engine);
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
