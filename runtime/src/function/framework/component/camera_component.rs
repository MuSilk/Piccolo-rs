use std::any::Any;

use crate::{
    core::math::{self, quaternion::Quaternion, vector3::Vector3},
    engine::Engine,
    function::{
        framework::{
            component::{character_component::CharacterComponent, component::ComponentTrait},
            object::object::GObject,
            resource::component::camera::{
                CameraComponentRes, CameraParameter, FirstPersonCameraParameter,
                FreeCameraParameter,
            },
        },
        input::game_command_system::{GameCommand, GameCommandInputSystem},
        render::{
            render_camera::RenderCameraType, render_swap_context::CameraSwapData,
            render_system::RenderSystem,
        },
    },
};

#[derive(Clone)]
pub enum CameraMode {
    ThirdPerson,
    FirstPerson,
    Free,
    Invalid,
}

#[derive(Clone)]
pub struct CameraComponent {
    pub m_camera_res: CameraComponentRes,

    m_camera_mode: CameraMode,

    pub m_position: Vector3,
    pub m_forward: Vector3,
    pub m_up: Vector3,
    pub m_left: Vector3,
}

impl CameraComponent {
    pub fn new_free_camera() -> Self {
        Self {
            m_camera_res: CameraComponentRes {
                m_parameter: CameraParameter::Free(FreeCameraParameter::default()),
            },
            m_camera_mode: CameraMode::Free,
            m_position: Default::default(),
            m_forward: Vector3::NEGATIVE_UNIT_Y,
            m_up: Vector3::UNIT_Z,
            m_left: Vector3::UNIT_X,
        }
    }

    pub fn new() -> Self {
        Self {
            m_camera_res: CameraComponentRes {
                m_parameter: CameraParameter::FirstPerson(FirstPersonCameraParameter::default()),
            },
            m_camera_mode: CameraMode::FirstPerson,
            m_position: Default::default(),
            m_forward: Vector3::NEGATIVE_UNIT_Y,
            m_up: Vector3::UNIT_Z,
            m_left: Vector3::UNIT_X,
        }
    }

    pub fn tick_first_person_camera(
        &mut self,
        input_system: &GameCommandInputSystem,
        render_system: &RenderSystem,
        character: &mut CharacterComponent,
    ) {
        let q_yaw = Quaternion::from_angle_axis(input_system.cursor_delta_yaw(), &Vector3::UNIT_Z);
        let q_pitch = Quaternion::from_angle_axis(input_system.cursor_delta_pitch(), &self.m_left);

        let (offset, h_eye) =
            if let CameraParameter::FirstPerson(param) = &self.m_camera_res.m_parameter {
                (param.m_vertical_offset, param.m_horizontal_eye_offset)
            } else {
                panic!("Invalid camera parameter");
            };

        self.m_position = character.get_position() + h_eye + Vector3::UNIT_Z * offset;

        self.m_forward = q_yaw * q_pitch * self.m_forward;
        self.m_left = q_yaw * q_pitch * self.m_left;
        self.m_up = self.m_forward.cross(&self.m_left);

        let desired_mat = math::look_at(
            &self.m_position,
            &(self.m_position + self.m_forward),
            &self.m_up,
        );

        render_system
            .get_logic_swap_data()
            .borrow_mut()
            .m_camera_swap_data = Some(CameraSwapData {
            m_fov_x: None,
            m_camera_type: Some(RenderCameraType::Motor),
            m_view_matrix: Some(desired_mat),
        });

        let object_facing = self.m_forward - Vector3::UNIT_Z * self.m_forward.dot(&Vector3::UNIT_Z);
        let object_left = Vector3::UNIT_Z.cross(&object_facing);
        let object_rotation =
            Quaternion::from_axes(&object_left, &-object_facing, &Vector3::UNIT_Z);
        character.set_rotation(object_rotation);
    }

    pub fn tick_third_person_camera(
        &mut self,
        input_system: &GameCommandInputSystem,
        render_system: &RenderSystem,
        character: &mut CharacterComponent,
    ) {
        let q_yaw = Quaternion::from_angle_axis(input_system.cursor_delta_yaw(), &Vector3::UNIT_Z);
        let q_pitch =
            Quaternion::from_angle_axis(input_system.cursor_delta_pitch(), &Vector3::UNIT_X);

        let (vertical_offset, horizontal_offset, param_m_cursor_pitch) =
            if let CameraParameter::ThirdPerson(param) = &mut self.m_camera_res.m_parameter {
                param.m_cursor_pitch = q_pitch * param.m_cursor_pitch;
                (
                    param.m_vertical_offset,
                    param.m_horizontal_offset,
                    param.m_cursor_pitch,
                )
            } else {
                panic!("Invalid camera parameter");
            };

        let offset = Vector3::new(0.0, horizontal_offset, vertical_offset);

        let center_pos = character.get_position() + Vector3::UNIT_Z * vertical_offset;
        self.m_position =
            character.get_rotation() * param_m_cursor_pitch * offset + character.get_position();

        self.m_forward = (center_pos - self.m_position).normalize();
        self.m_up = character.get_rotation() * param_m_cursor_pitch * Vector3::UNIT_Z;
        self.m_left = self.m_up.cross(&self.m_forward);

        character.set_rotation(q_yaw * character.get_rotation());

        let desired_mat = math::look_at(
            &self.m_position,
            &(self.m_position + self.m_forward),
            &self.m_up,
        );
        render_system
            .get_logic_swap_data()
            .borrow_mut()
            .m_camera_swap_data = Some(CameraSwapData {
            m_fov_x: None,
            m_camera_type: Some(RenderCameraType::Motor),
            m_view_matrix: Some(desired_mat),
        });
    }

    pub fn tick_free_camera(
        &mut self,
        input_system: &GameCommandInputSystem,
        render_system: &RenderSystem,
        delta_time: f32,
    ) {
        let command = input_system.get_game_command();
        if command.contains(GameCommand::invalid) {
            return;
        }

        let q_yaw = Quaternion::from_angle_axis(input_system.cursor_delta_yaw(), &Vector3::UNIT_Z);
        let q_pitch = Quaternion::from_angle_axis(input_system.cursor_delta_pitch(), &self.m_left);

        self.m_forward = q_yaw * q_pitch * self.m_forward;
        self.m_left = q_yaw * q_pitch * self.m_left;
        self.m_up = self.m_forward.cross(&self.m_left);

        if command.intersects(
            GameCommand::forward
                | GameCommand::backward
                | GameCommand::left
                | GameCommand::right
                | GameCommand::up
                | GameCommand::down,
        ) {
            let mut move_direction = Vector3::ZERO;
            if command.contains(GameCommand::forward) {
                move_direction += self.m_forward;
            }
            if command.contains(GameCommand::backward) {
                move_direction -= self.m_forward;
            }
            if command.contains(GameCommand::left) {
                move_direction += self.m_left;
            }
            if command.contains(GameCommand::right) {
                move_direction -= self.m_left;
            }
            if command.contains(GameCommand::up) {
                move_direction += self.m_up;
            }
            if command.contains(GameCommand::down) {
                move_direction -= self.m_up;
            }
            self.m_position += move_direction * 2.0 * delta_time;
        }

        let desired_mat = math::look_at(
            &self.m_position,
            &(self.m_position + self.m_forward),
            &self.m_up,
        );
        render_system
            .get_logic_swap_data()
            .borrow_mut()
            .m_camera_swap_data = Some(CameraSwapData {
            m_fov_x: None,
            m_camera_type: Some(RenderCameraType::Motor),
            m_view_matrix: Some(desired_mat),
        });
    }

    pub fn look_at(&mut self, position: Vector3, target: &Vector3, up: &Vector3) {
        self.m_position = position;
        let forward = (target - position).normalize();

        let right = forward.cross(&up.normalize()).normalize();

        self.m_forward = forward;
        self.m_left = -right;
        self.m_up = right.cross(&forward);
    }
}

impl ComponentTrait for CameraComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn tick(&mut self, engine: &Engine, gobject: &GObject, delta_time: f32) {
        match self.m_camera_mode {
            CameraMode::FirstPerson => self.tick_first_person_camera(
                &engine.input_system().borrow(),
                &engine.render_system().borrow(),
                &mut gobject.get_component_mut::<CharacterComponent>().unwrap(),
            ),
            CameraMode::ThirdPerson => self.tick_third_person_camera(
                &engine.input_system().borrow(),
                &engine.render_system().borrow(),
                &mut gobject.get_component_mut::<CharacterComponent>().unwrap(),
            ),
            CameraMode::Free => self.tick_free_camera(
                &engine.input_system().borrow(),
                &engine.render_system().borrow(),
                delta_time,
            ),
            CameraMode::Invalid => panic!("Invalid camera mode"),
        }
    }
}
