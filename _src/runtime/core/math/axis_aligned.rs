use std::f32;

use nalgebra_glm::Vec3;

pub struct AxisAlignedBox {
    m_center: Vec3,
    m_half_extent : Vec3,
    m_min_corner: Vec3,
    m_max_corner: Vec3
}

impl Default for AxisAlignedBox {
    fn default() -> Self {
        AxisAlignedBox { 
            m_center: Vec3::new(0.0, 0.0, 0.0), 
            m_half_extent: Vec3::new(0.0, 0.0, 0.0), 
            m_min_corner: Vec3::new(f32::MAX, f32::MAX, f32::MAX), 
            m_max_corner: Vec3::new(f32::MIN, f32::MIN, f32::MIN)
        }
    }
}

impl AxisAlignedBox {
    pub fn new(center: &Vec3, half_extent: &Vec3) -> Self {
        AxisAlignedBox { 
            m_center: *center, 
            m_half_extent: *half_extent, 
            ..Default::default()
        }
    }

    pub fn merge(&mut self,new_point: &Vec3){
        self.m_min_corner = Vec3::new(
            self.m_min_corner.x.min(new_point.x),
            self.m_min_corner.y.min(new_point.y),
            self.m_min_corner.z.min(new_point.z),
        );
        self.m_max_corner = Vec3::new(
            self.m_max_corner.x.max(new_point.x),
            self.m_max_corner.y.max(new_point.y),
            self.m_max_corner.z.max(new_point.z),
        );
        self.m_center = (self.m_min_corner + self.m_max_corner) * 0.5;
        self.m_half_extent = self.m_center - self.m_min_corner;
    }

    pub fn update(&mut self, center: &Vec3, half_extent: &Vec3){
        self.m_center = *center;
        self.m_half_extent = *half_extent;
        self.m_min_corner = self.m_center - self.m_half_extent;
        self.m_max_corner = self.m_center + self.m_half_extent;
    }

    pub fn get_center(&self) -> &Vec3 {
        &self.m_center
    }

    pub fn get_half_extent(&self) -> &Vec3 {
        &self.m_half_extent
    }

    pub fn get_min_corner(&self) -> &Vec3 {
        &self.m_min_corner
    }

    pub fn get_max_corner(&self) -> &Vec3 {
        &self.m_max_corner
    }

}