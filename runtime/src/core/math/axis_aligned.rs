use std::f32;

use crate::core::math::vector3::Vector3;

#[derive(Clone, Debug)]
pub struct AxisAlignedBox {
    m_center: Vector3,
    m_half_extent : Vector3,
    m_min_corner: Vector3,
    m_max_corner: Vector3
}

impl Default for AxisAlignedBox {
    fn default() -> Self {
        AxisAlignedBox { 
            m_center: Vector3::new(0.0, 0.0, 0.0), 
            m_half_extent: Vector3::new(0.0, 0.0, 0.0), 
            m_min_corner: Vector3::new(f32::MAX, f32::MAX, f32::MAX), 
            m_max_corner: Vector3::new(f32::MIN, f32::MIN, f32::MIN)
        }
    }
}

impl AxisAlignedBox {
    pub fn new(center: Vector3, half_extent: Vector3) -> Self {
        AxisAlignedBox { 
            m_center: center, 
            m_half_extent: half_extent, 
            m_min_corner: center - half_extent,
            m_max_corner: center + half_extent,
        }
    }

    pub fn merge(&mut self,new_point: &Vector3){
        self.m_min_corner = Vector3::new(
            self.m_min_corner.x.min(new_point.x),
            self.m_min_corner.y.min(new_point.y),
            self.m_min_corner.z.min(new_point.z),
        );
        self.m_max_corner = Vector3::new(
            self.m_max_corner.x.max(new_point.x),
            self.m_max_corner.y.max(new_point.y),
            self.m_max_corner.z.max(new_point.z),
        );
        self.m_center = (self.m_min_corner + self.m_max_corner) * 0.5;
        self.m_half_extent = self.m_center - self.m_min_corner;
    }

    pub fn update(&mut self, center: &Vector3, half_extent: &Vector3){
        self.m_center = *center;
        self.m_half_extent = *half_extent;
        self.m_min_corner = self.m_center - self.m_half_extent;
        self.m_max_corner = self.m_center + self.m_half_extent;
    }

    pub fn get_center(&self) -> &Vector3 {
        &self.m_center
    }

    pub fn get_half_extent(&self) -> &Vector3 {
        &self.m_half_extent
    }

    pub fn get_min_corner(&self) -> &Vector3 {
        &self.m_min_corner
    }

    pub fn get_max_corner(&self) -> &Vector3 {
        &self.m_max_corner
    }

}