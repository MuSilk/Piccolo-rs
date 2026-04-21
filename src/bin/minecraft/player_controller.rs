//! 第一人称移动与轴对齐盒碰撞（独立实现，不依赖 `minecraft::ecs`）。

use std::{cell::RefCell, rc::Rc};

use runtime::{
    core::math::{axis_aligned::AxisAlignedBox, vector3::Vector3},
    function::framework::component::motor_component::Controller,
};

use crate::voxel_world::VoxelWorld;

/// 水平半宽略大于原版 0.3，减轻贴墙时相机穿模。
pub struct AiPlayerController {
    half_extent: Vector3,
    world: Rc<RefCell<Box<VoxelWorld>>>,
}

impl AiPlayerController {
    pub fn new(world: Rc<RefCell<Box<VoxelWorld>>>) -> Self {
        Self {
            world,
            half_extent: Vector3::new(0.34, 0.34, 0.9),
        }
    }
}

impl Controller for AiPlayerController {
    fn r#move(&self, current_position: &Vector3, displacement: &Vector3) -> Vector3 {
        let try_full = *current_position + *displacement;
        let probe = AxisAlignedBox::new(try_full + self.half_extent, self.half_extent);
        if self.world.borrow().collect_block_hits(&probe).is_empty() {
            return try_full;
        }

        let mut p = *current_position;
        p.x += displacement.x;
        if !self
            .world
            .borrow()
            .collect_block_hits(&AxisAlignedBox::new(p + self.half_extent, self.half_extent))
            .is_empty()
        {
            p.x = current_position.x;
        }
        p.y += displacement.y;
        if !self
            .world
            .borrow()
            .collect_block_hits(&AxisAlignedBox::new(p + self.half_extent, self.half_extent))
            .is_empty()
        {
            p.y = current_position.y;
        }
        p.z += displacement.z;
        if !self
            .world
            .borrow()
            .collect_block_hits(&AxisAlignedBox::new(p + self.half_extent, self.half_extent))
            .is_empty()
        {
            p.z = current_position.z;
        }
        p
    }
}
