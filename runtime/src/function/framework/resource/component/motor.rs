use serde::{Deserialize, Serialize};

use crate::function::framework::resource::resource::Resource;


#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MotorComponentRes {
    pub move_speed: f32,
    pub jump_height: f32,
    pub max_move_speed_ratio: f32,
    pub max_sprint_speed_ratio: f32,
    pub move_acceleration: f32,
    pub sprint_acceleration: f32,
}

#[typetag::serde]
impl Resource for MotorComponentRes {

}