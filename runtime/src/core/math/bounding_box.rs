use crate::core::math::{matrix4::Matrix4x4, vector3::Vector3};

pub struct BoundingBox {
    pub min_bound: Vector3,
    pub max_bound: Vector3,
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            min_bound: Vector3::new(f32::MAX, f32::MAX, f32::MAX),
            max_bound: Vector3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }
}

impl BoundingBox {
    pub fn merge(&mut self, point: Vector3) {
        self.min_bound.x = self.min_bound.x.min(point.x);
        self.min_bound.y = self.min_bound.y.min(point.y);
        self.min_bound.z = self.min_bound.z.min(point.z);

        self.max_bound.x = self.max_bound.x.max(point.x);
        self.max_bound.y = self.max_bound.y.max(point.y);
        self.max_bound.z = self.max_bound.z.max(point.z);
    }
    pub fn merge_box(&mut self, other: &BoundingBox) {
        self.min_bound.x = self.min_bound.x.min(other.min_bound.x);
        self.min_bound.y = self.min_bound.y.min(other.min_bound.y);
        self.min_bound.z = self.min_bound.z.min(other.min_bound.z);

        self.max_bound.x = self.max_bound.x.max(other.max_bound.x);
        self.max_bound.y = self.max_bound.y.max(other.max_bound.y);
        self.max_bound.z = self.max_bound.z.max(other.max_bound.z);
    }
}

pub fn bounding_box_transform(b: &BoundingBox, m: &Matrix4x4) -> BoundingBox { 
    let g_box_offset = [
        Vector3::new(-1.0, -1.0,  1.0),
        Vector3::new( 1.0, -1.0,  1.0),
        Vector3::new( 1.0,  1.0,  1.0),
        Vector3::new(-1.0,  1.0,  1.0),
        Vector3::new(-1.0, -1.0, -1.0),
        Vector3::new( 1.0, -1.0, -1.0),
        Vector3::new( 1.0,  1.0, -1.0),
        Vector3::new(-1.0,  1.0, -1.0),
    ];
    let center = (b.max_bound + b.min_bound) * 0.5;
    let extent = (b.max_bound - b.min_bound) * 0.5;
    let mut result = BoundingBox::default();
    for i in 0..8 {
        let corner_before = extent * g_box_offset[i] + center;
        let corner_with_w = m * corner_before.to_homogeneous();
        let corner = Vector3::from_homogeneous(&corner_with_w);
        result.merge(corner);
    }
    result
}