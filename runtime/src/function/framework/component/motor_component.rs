use crate::{core::math::{quaternion::Quaternion, vector3::Vector3}, function::{framework::{component::{component::{Component, ComponentTrait}, transform_component::TransformComponent}, resource::component::motor::MotorComponentRes}, global::global_context::RuntimeGlobalContext, input::input_system::GameCommand}};

pub trait Controller{
    fn r#move(&self, current_position: &Vector3, displacement: &Vector3) -> Vector3;
}


pub struct MotorComponent {
    m_component: Component,

    m_motor_res: MotorComponentRes,
    
    m_move_speed_ratio: f32,
    m_vertical_move_speed: f32,
    m_jump_horizontal_speed_ratio: f32,

    m_desired_displacement: Vector3,
    m_desired_horizontal_move_direction: Vector3,
    m_jump_initial_velocity: Vector3,
    m_target_position: Vector3,

    m_controller: Box<dyn Controller>,

    m_is_moving: bool,
    m_is_landing: bool,
}

impl ComponentTrait for MotorComponent {
    fn get_component(&self) -> &Component {
        &self.m_component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.m_component
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl MotorComponent {

    pub fn new<T: 'static + Controller>(controller: Box<T>) -> Self {
        Self {
            m_component: Component::default(),

            m_motor_res: MotorComponentRes::default(),
            
            m_move_speed_ratio: 0.0,
            m_vertical_move_speed: 0.0,
            m_jump_horizontal_speed_ratio: 0.0,

            m_desired_displacement: Vector3::default(),
            m_desired_horizontal_move_direction: Vector3::default(),
            m_jump_initial_velocity: Vector3::default(),
            m_target_position: Vector3::default(),

            m_controller: controller,

            m_is_moving: false,
            m_is_landing: false,
        }
    }

    pub fn post_load_resources(&mut self, motor_res: &MotorComponentRes) {
        self.m_motor_res = motor_res.clone();
    }
    pub fn get_is_moving(&self) -> bool {
        self.m_is_moving
    }

    pub fn get_target_position(&self) -> &Vector3 {
        &self.m_target_position
    }

    pub fn tick(&mut self, delta_time: f32, transform: &mut TransformComponent) {
        let input_system = RuntimeGlobalContext::get_input_system().borrow();
        let command = input_system.get_game_command();

        if command.contains(GameCommand::invalid) {
            return;
        }
        self.calculate_desired_horizontal_move_speed(delta_time, command);
        self.calculate_desired_vertical_move_speed(delta_time, command);
        self.calculate_desired_move_direction(command, transform.get_rotation());
        self.calculate_desired_displacement(delta_time);
        self.calculate_target_position(transform.get_position());

        transform.set_position(self.m_target_position);
    }
}

impl MotorComponent {
    fn calculate_desired_horizontal_move_speed(&mut self, delta_time: f32, command: &GameCommand) {
        let has_move_command = 
            command.intersects(GameCommand::forward | GameCommand::backward | GameCommand::left | GameCommand::right) &&
            !command.contains(GameCommand::free_camera);
        let has_sprint_command = command.contains(GameCommand::sprint);

        let (is_acceleration, min_speed_ratio, max_speed_ratio, final_acceleration) =
            if has_move_command && self.m_move_speed_ratio >= self.m_motor_res.max_move_speed_ratio {
                (
                    has_sprint_command,
                    self.m_motor_res.max_move_speed_ratio, 
                    self.m_motor_res.max_sprint_speed_ratio,
                    self.m_motor_res.sprint_acceleration
                )
            }
            else if has_move_command{
                (true, 0.0, self.m_motor_res.max_move_speed_ratio, self.m_motor_res.move_acceleration)
            }
            else{
                (false, 0.0, self.m_motor_res.max_sprint_speed_ratio, self.m_motor_res.move_acceleration)
            };
        self.m_move_speed_ratio += if is_acceleration {1.0} else {-1.0} * final_acceleration * delta_time;
        self.m_move_speed_ratio = self.m_move_speed_ratio.clamp(min_speed_ratio, max_speed_ratio);
    }

    fn calculate_desired_vertical_move_speed(&mut self, delta_time: f32, command: &GameCommand) {
        if self.m_motor_res.jump_height == 0.0 {
            return;
        }
        let gravity = 9.8;//todo: configable gravity

        if self.m_is_landing {
            if command.contains(GameCommand::jump) {
                self.m_is_landing = false;
                self.m_vertical_move_speed = (self.m_motor_res.jump_height * 2.0 * gravity).sqrt();
                self.m_jump_horizontal_speed_ratio = self.m_move_speed_ratio; 
            }
            else{
                self.m_vertical_move_speed = -gravity * delta_time;
            }
        }
        else{
            self.m_vertical_move_speed -= gravity * delta_time;
        }
    }

    fn calculate_desired_move_direction(&mut self, command: &GameCommand, rotation: &Quaternion) {
        if !self.m_is_landing { 
            let forward_dir = rotation * Vector3::NEGATIVE_UNIT_Y;
            let left_dir = rotation * Vector3::UNIT_X;

            if !command.is_empty() {
                self.m_desired_horizontal_move_direction = Vector3::ZERO;
            }

            if command.contains(GameCommand::forward) {
                self.m_desired_horizontal_move_direction += forward_dir;
            }

            if command.contains(GameCommand::backward) {
                self.m_desired_horizontal_move_direction -= forward_dir;
            }

            if command.contains(GameCommand::left) {
                self.m_desired_horizontal_move_direction += left_dir;
            }

            if command.contains(GameCommand::right) {
                self.m_desired_horizontal_move_direction -= left_dir;
            }
            self.m_desired_horizontal_move_direction = self.m_desired_horizontal_move_direction.normalize();
        }
    }

    fn calculate_desired_displacement(&mut self, delta_time: f32) {
        let horizontal_speed_ratio = if self.m_is_landing {
            self.m_move_speed_ratio
        } else {
            self.m_jump_horizontal_speed_ratio
        };
        self.m_desired_displacement = 
            self.m_desired_horizontal_move_direction * self.m_motor_res.move_speed * horizontal_speed_ratio * delta_time +
            Vector3::UNIT_Z * self.m_vertical_move_speed * delta_time;
    }

    fn calculate_target_position(&mut self, position: &Vector3) {
        let mut final_position = self.m_controller.r#move(position, &self.m_desired_displacement);
        // character always above z-plane

        if self.m_is_landing {
            if final_position.z != position.z {
                self.m_is_landing = false;
            }
        }
        else{
            if final_position.z + self.m_desired_displacement.z <= 0.0 {
                final_position.z = 0.0;
                self.m_is_landing = true;
            }
            else if final_position.z == position.z {
                self.m_is_landing = true;
            }
        }

        self.m_is_moving = (final_position - position).squared_length() > 0.0;
        self.m_target_position = final_position;
    }
}