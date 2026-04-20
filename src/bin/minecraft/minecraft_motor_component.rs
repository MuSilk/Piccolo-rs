//! 面向方块世界的移动：水平速度 + 地面快速贴目标速度、空中弱加速度，手感接近 Java 版步行/疾跑/起跳。
//!
//! 仍使用 `MotorComponentRes` 与 `player.motor.json`：`move_speed`、`max_*_speed_ratio` 控制走/跑标量，`jump_height` 控制起跳初速度。
//!
//! 碰撞与地形采样一致，不依赖体素网格是否已提交渲染；但首帧或断点恢复时 `delta_time` 可能极大，若不限制单步积分会出现大位移穿透 AABB 检测，表现为「一出生就在地下」。

use runtime::{
    core::math::{quaternion::Quaternion, vector3::Vector3},
    function::{
        framework::{component::{
            component::ComponentTrait,
            motor_component::Controller,
            transform_component::TransformComponent,
        }, resource::component::motor::MotorComponentRes},
        input::{game_command_system::{GameCommand, GameCommandInputSystem}},
    },
};

/// 地面水平加速度（单位/秒²），略高更「跟手」。
const GROUND_ACCEL: f32 = 38.0;
/// 无输入时水平速度衰减（越大停得越快）。
const GROUND_FRICTION: f32 = 18.0;
/// 空中沿输入方向的额外加速度（明显弱于地面）。
const AIR_ACCEL: f32 = 1.05;
/// 空中水平速度上限相对疾跑速度的倍数（保留起跳水平动量）。
const AIR_SPEED_CAP_MUL: f32 = 1.12;
const GRAVITY: f32 = 16.0;
const VEL_EPS: f32 = 1e-5;
/// 单帧用于积分/摩擦的最大步长（秒），避免 `delta_time` 尖峰导致竖直位移穿透碰撞体。
const MAX_PHYSICS_DT: f32 = 1.0 / 30.0;

pub struct MinecraftMotorComponent {
    m_motor_res: MotorComponentRes,
    m_horizontal_vel: Vector3,
    m_vertical_vel: f32,
    m_target_position: Vector3,
    m_is_moving: bool,
    m_is_landing: bool,
    m_controller: Box<dyn Controller>,
}

impl ComponentTrait for MinecraftMotorComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MinecraftMotorComponent {
    pub fn new<T: 'static + Controller>(controller: Box<T>) -> Self {
        Self {
            m_motor_res: MotorComponentRes::default(),
            m_horizontal_vel: Vector3::ZERO,
            m_vertical_vel: 0.0,
            m_target_position: Vector3::ZERO,
            m_is_moving: false,
            m_is_landing: false,
            m_controller: controller,
        }
    }

    pub fn post_load_resources(&mut self, motor_res: &MotorComponentRes) {
        self.m_motor_res = motor_res.clone();
    }

    /// 与 `Transform`/角色初始位置对齐，避免首帧或 `invalid` 输入时 `m_target_position` 仍为原点（地表以下全是固体）。
    pub fn align_spawn(&mut self, position: Vector3) {
        self.m_target_position = position;
        self.m_horizontal_vel = Vector3::ZERO;
        self.m_vertical_vel = 0.0;
        self.m_is_landing = true;
    }

    pub fn get_is_moving(&self) -> bool {
        self.m_is_moving
    }

    pub fn get_target_position(&self) -> &Vector3 {
        &self.m_target_position
    }

    pub fn tick(
        &mut self,
        input_system: &GameCommandInputSystem,
        delta_time: f32,
        transform: &mut TransformComponent,
        facing_rotation: Quaternion,
    ) {
        let command = input_system.get_game_command();
        if command.contains(GameCommand::invalid) {
            self.m_target_position = *transform.get_position();
            return;
        }

        let dt = delta_time.max(0.0).min(MAX_PHYSICS_DT);

        let walk_speed = self.m_motor_res.move_speed * self.m_motor_res.max_move_speed_ratio;
        let sprint_speed = self.m_motor_res.move_speed * self.m_motor_res.max_sprint_speed_ratio;

        let wish_dir = self.wish_direction(command, &facing_rotation);
        let has_wish = wish_dir.squared_length() > VEL_EPS;
        let has_move_input = has_wish && !command.contains(GameCommand::free_camera);
        let sprinting = command.contains(GameCommand::sprint) && has_move_input;

        let target_speed = if !has_move_input {
            0.0
        } else if sprinting {
            sprint_speed
        } else {
            walk_speed
        };

        if self.m_is_landing {
            self.tick_vertical_ground(dt, command);
        } else {
            self.tick_vertical_air(dt);
        }

        if self.m_is_landing {
            let wish_vel = Vector3::new(
                wish_dir.x * target_speed,
                wish_dir.y * target_speed,
                0.0,
            );
            if has_move_input {
                self.m_horizontal_vel = accel_toward_xy(
                    self.m_horizontal_vel,
                    wish_vel,
                    GROUND_ACCEL,
                    dt,
                );
            } else {
                self.m_horizontal_vel =
                    apply_ground_friction_xy(self.m_horizontal_vel, GROUND_FRICTION, dt);
            }
        } else if has_move_input {
            let add = Vector3::new(wish_dir.x, wish_dir.y, 0.0) * (AIR_ACCEL * dt);
            self.m_horizontal_vel = self.m_horizontal_vel + add;
            let cap = sprint_speed * AIR_SPEED_CAP_MUL;
            let h = xy(self.m_horizontal_vel);
            let len = h.length();
            if len > cap && len > VEL_EPS {
                let scale = cap / len;
                self.m_horizontal_vel.x *= scale;
                self.m_horizontal_vel.y *= scale;
            }
        }

        let disp = Vector3::new(
            self.m_horizontal_vel.x * dt,
            self.m_horizontal_vel.y * dt,
            self.m_vertical_vel * dt,
        );
        self.apply_position(transform.get_position(), &disp);
        transform.set_position(self.m_target_position);
    }

    fn wish_direction(&self, command: &GameCommand, rotation: &Quaternion) -> Vector3 {
        // 只用水平面内的朝向（忽略俯仰），避免低头时把 forward 压到 XY 后方向乱飘或接近零。
        let forward_full = *rotation * Vector3::NEGATIVE_UNIT_Y;
        let mut forward_xy = Vector3::new(forward_full.x, forward_full.y, 0.0);
        if forward_xy.squared_length() < VEL_EPS {
            forward_xy = Vector3::new(0.0, -1.0, 0.0);
        } else {
            forward_xy = forward_xy.normalize();
        }
        let mut left_xy = Vector3::UNIT_Z.cross(&forward_xy);
        if left_xy.squared_length() < VEL_EPS {
            left_xy = Vector3::UNIT_X;
        } else {
            left_xy = left_xy.normalize();
        }

        let mut w = Vector3::ZERO;
        if command.contains(GameCommand::forward) {
            w = w + forward_xy;
        }
        if command.contains(GameCommand::backward) {
            w = w - forward_xy;
        }
        if command.contains(GameCommand::left) {
            w = w + left_xy;
        }
        if command.contains(GameCommand::right) {
            w = w - left_xy;
        }
        let h = xy(w);
        let len_sq = h.squared_length();
        if len_sq < VEL_EPS {
            Vector3::ZERO
        } else {
            let inv = 1.0 / len_sq.sqrt();
            Vector3::new(h.x * inv, h.y * inv, 0.0)
        }
    }

    fn tick_vertical_ground(&mut self, delta_time: f32, command: &GameCommand) {
        if self.m_motor_res.jump_height <= 0.0 {
            return;
        }
        if command.contains(GameCommand::jump) {
            self.m_is_landing = false;
            self.m_vertical_vel = (self.m_motor_res.jump_height * 2.0 * GRAVITY).sqrt();
        } else {
            self.m_vertical_vel = -GRAVITY * delta_time;
        }
    }

    fn tick_vertical_air(&mut self, delta_time: f32) {
        if self.m_motor_res.jump_height <= 0.0 {
            return;
        }
        self.m_vertical_vel -= GRAVITY * delta_time;
    }

    fn apply_position(&mut self, position: &Vector3, displacement: &Vector3) {
        let mut final_position = self.m_controller.r#move(position, displacement);

        if self.m_is_landing {
            if final_position.z != position.z {
                self.m_is_landing = false;
            }
        } else {
            if final_position.z + displacement.z <= 0.0 {
                final_position.z = 0.0;
                self.m_is_landing = true;
            } else if final_position.z == position.z {
                self.m_is_landing = true;
            }
        }

        self.m_is_moving = (final_position - position).squared_length() > 0.0;
        self.m_target_position = final_position;
    }
}

fn xy(v: Vector3) -> Vector3 {
    Vector3::new(v.x, v.y, 0.0)
}

fn accel_toward_xy(current: Vector3, target: Vector3, max_accel: f32, dt: f32) -> Vector3 {
    let diff = target - current;
    let len = diff.length();
    if len < VEL_EPS {
        return target;
    }
    let step = max_accel * dt;
    if len <= step {
        target
    } else {
        current + diff * (step / len)
    }
}

fn apply_ground_friction_xy(mut v: Vector3, friction: f32, dt: f32) -> Vector3 {
    let h = xy(v);
    let len = h.length();
    if len < VEL_EPS {
        return Vector3::ZERO;
    }
    let drop = friction * dt;
    let new_len = (len - drop).max(0.0);
    if new_len < VEL_EPS {
        Vector3::ZERO
    } else {
        let s = new_len / len;
        v.x *= s;
        v.y *= s;
        v
    }
}
