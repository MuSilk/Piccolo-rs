use std::{cell::RefCell, rc::Rc};

use runtime::{core::math::{axis_aligned::AxisAlignedBox, vector3::Vector3}, function::framework::component::motor_component::Controller};

use crate::world::World;

pub struct CharacterController{
    half_extent: Vector3,
    world: Rc<RefCell<Box<World>>>,
}   

impl CharacterController {
    pub fn new(world: Rc<RefCell<Box<World>>>) -> Self {
        Self { world , half_extent: Vector3::new(0.2, 0.2, 0.9) }
    }
}

impl Controller for CharacterController {
    fn r#move(&self, current_position: &Vector3, displacement: &Vector3) -> Vector3 {
        let final_position = current_position + displacement;
        // println!("{:?} {:?}",current_position, final_position);
        let area = AxisAlignedBox::new(
            final_position + self.half_extent,
            self.half_extent
        );
        if self.world.borrow().get_aabbs(&area).is_empty() {
            return final_position;
        }

        let mut adjusted_position = *current_position;

        // 在X轴上移动
        adjusted_position.x += displacement.x;
        let x_aabb = AxisAlignedBox::new(
            adjusted_position + self.half_extent,
            self.half_extent
        );
        if !self.world.borrow().get_aabbs(&x_aabb).is_empty() {
            adjusted_position.x = current_position.x; // 回退X轴移动
        }

        // 在Y轴上移动
        adjusted_position.y += displacement.y;
        let y_aabb = AxisAlignedBox::new(
            adjusted_position + self.half_extent,
            self.half_extent
        );
        if !self.world.borrow().get_aabbs(&y_aabb).is_empty() {
            adjusted_position.y = current_position.y; // 回退Y轴移动
        }

        // 在Z轴上移动
        adjusted_position.z += displacement.z;
        let z_aabb = AxisAlignedBox::new(
            adjusted_position + self.half_extent,
            self.half_extent
        );
        if !self.world.borrow().get_aabbs(&z_aabb).is_empty() {
            adjusted_position.z = current_position.z; // 回退Z轴移动
        }

        adjusted_position
    }
}