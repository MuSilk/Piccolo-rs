use crate::{core::math::vector3::Vector3, function::framework::component::component::Component};

struct CharacterController{

}

enum MotorState {
    Moving,
    Jumping,
}

enum JumpState {
    Idle,
    Rising,
    Falling,
}

pub struct MotorComponent {
    m_component: Component,
    
    m_move_speed_ratio: f32,
    m_vertical_move_speed: f32,
    m_jump_horizontal_speed_ratio: f32,

    m_desired_displacement: Vector3,
    m_desired_horizontal_move_direction: Vector3,
    m_jump_initial_velocity: Vector3,
    m_target_position: Vector3,

    m_motor_state: MotorState,
    m_jump_state: JumpState,
    m_controller: CharacterController,

    m_is_moving: bool,
}

impl MotorComponent {
    
}