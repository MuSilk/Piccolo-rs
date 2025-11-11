use std::{f32::consts::PI, ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign}};

use serde::{Deserialize, Serialize};

use crate::core::math::{matrix4::Matrix4x4, quaternion::Quaternion, vector4::Vector4};


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const UNIT_X: Vector3 = Vector3::new(1.0, 0.0, 0.0);
    pub const UNIT_Y: Vector3 = Vector3::new(0.0, 1.0, 0.0);
    pub const UNIT_Z: Vector3 = Vector3::new(0.0, 0.0, 1.0);
    pub const NEGATIVE_UNIT_X: Vector3 = Vector3::new(-1.0, 0.0, 0.0);
    pub const NEGATIVE_UNIT_Y: Vector3 = Vector3::new(0.0, -1.0, 0.0);
    pub const NEGATIVE_UNIT_Z: Vector3 = Vector3::new(0.0, 0.0, -1.0);
    pub const ZERO:   Vector3 = Vector3::new(0.0, 0.0, 0.0);
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub const fn one() -> Self {
        Vector3::new(1.0, 1.0, 1.0)
    }

    pub const fn zero() -> Self {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub const fn from_homogeneous(v: &Vector4) -> Vector3 {
        Vector3::new(v.x / v.w, v.y / v.w, v.z / v.w)
    }

    pub const fn to_translate_matrix(&self) -> Matrix4x4 {
        Matrix4x4::from_columns(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [self.x, self.y, self.z, 1.0],
        )
    }

    pub const fn to_scale_matrix(&self) -> Matrix4x4 {
        Matrix4x4::from_columns(
            [self.x, 0.0, 0.0, 0.0],
            [0.0, self.y, 0.0, 0.0],
            [0.0, 0.0, self.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub const fn to_homogeneous(&self) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, 1.0)
    }

    pub const fn cross(&self, rhs: &Vector3) -> Vector3 {
        Vector3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub const fn dot(&self, rhs: &Vector3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn is_zero(&self) -> bool {
        (self.x * self.x + self.y * self.y + self.z * self.z) < 1e-12
    }

    pub fn normalize(&self) -> Vector3 {
        let len_inv = 1.0 / self.length();
        Vector3 {
            x: self.x * len_inv,
            y: self.y * len_inv,
            z: self.z * len_inv,
        }
    }

    pub fn get_rotation_to(&self, dest: &Vector3) -> Quaternion {
        let v0 = self.normalize();
        let v1 = dest.normalize();
        let d = v0.dot(&v1);
        if d >= 1.0 {
            Quaternion::identity()
        } 
        else if d < 1e-6 - 1.0 {
            let mut axis = Vector3::UNIT_X.cross(self);
            if axis.is_zero() {
                axis = Vector3::UNIT_Y.cross(self);
            }
            axis = axis.normalize();
            Quaternion::from_angle_axis(PI, &axis)
        }
        else{
            let s = (1.0 + d).sqrt() * 2.0;
            let invs = 1.0 / s;
            let c = v0.cross(&v1);
            Quaternion::new(
                c.x * invs,
                c.y * invs,
                c.z * invs,
                s * 0.5,
            ).normalize()
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl Neg for &Vector3 {
    type Output = Vector3;
    fn neg(self) -> Self::Output {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl Add<Vector3> for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Add<&Vector3> for Vector3 {
    type Output = Self;
    fn add(self, rhs: &Self) -> Self::Output {
        Vector3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Add<Vector3> for &Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Self::Output {
        Vector3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Add<&Vector3> for &Vector3 {
    type Output = Vector3;
    fn add(self, rhs: &Vector3) -> Self::Output {
        Vector3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Sub<Vector3> for &Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: Vector3) -> Self::Output {
        Vector3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Sub<&Vector3> for Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: &Vector3) -> Self::Output {
        Vector3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Sub<&Vector3> for &Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: &Vector3) -> Self::Output {
        Vector3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Vector3> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: Vector3) -> Self::Output {
        Vector3 { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Vector3 { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl AddAssign<Vector3> for Vector3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<&Vector3> for Vector3 {
    fn add_assign(&mut self, rhs: &Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign<Vector3> for Vector3 {
    fn sub_assign(&mut self, rhs: Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<&Vector3> for Vector3 {
    fn sub_assign(&mut self, rhs: &Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}


impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
