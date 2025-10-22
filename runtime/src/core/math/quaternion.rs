use std::ops::Mul;
use crate::core::math::{matrix4::Matrix4x4, vector3::Vector3};


#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Quaternion {

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quaternion { x, y, z, w }
    }
    pub const fn identity() -> Self {
        Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }

    pub const fn to_rotation_matrix(&self) -> Matrix4x4 {
        let (x, y, z, w) = (self.x,self.y,self.z,self.w);
        let f_tx  = x + x;   // 2x
        let f_ty  = y + y;   // 2y
        let f_tz  = z + z;   // 2z
        let f_twx = f_tx * w; // 2xw
        let f_twy = f_ty * w; // 2yw
        let f_twz = f_tz * w; // 2z2
        let f_txx = f_tx * x; // 2x^2
        let f_txy = f_ty * x; // 2xy
        let f_txz = f_tz * x; // 2xz
        let f_tyy = f_ty * y; // 2y^2
        let f_tyz = f_tz * y; // 2yz
        let f_tzz = f_tz * z; // 2z^2

        Matrix4x4::from_columns(
            [1.0 - (f_tyy + f_tzz), f_txy + f_twz,          f_txz - f_twy,          0.0],
            [f_txy - f_twz,         1.0 - (f_txx + f_tzz),  f_tyz + f_twx,          0.0],
            [f_txz + f_twy,         f_tyz - f_twx,          1.0 - (f_txx + f_tyy),  0.0],
            [0.0,                   0.0,                    0.0,                    1.0],
        )
    }

    pub const fn conjugate(&self) -> Self {
        Quaternion { x: -self.x, y: -self.y, z: -self.z, w: self.w }
    }
    pub fn from_angle_axis(angle: f32, axis: &Vector3) -> Self {
        let half_angle = angle * 0.5;
        let (s, c) = half_angle.sin_cos();
        Quaternion {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: c,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let len_inv = 1.0 / self.length();
        Quaternion {
            x: self.x * len_inv,
            y: self.y * len_inv,
            z: self.z * len_inv,
            w: self.w * len_inv,
        }
    }
}

impl Mul<&Vector3> for Quaternion {
    type Output = Vector3;

    fn mul(self, rhs: &Vector3) -> Self::Output {
        let qvec = Vector3::new(self.x, self.y, self.z);
        let mut uv = qvec.cross(&rhs);
        let mut uuv = qvec.cross(&uv);
        uv *= 2.0 * self.w;
        uuv *= 2.0;
        
        rhs + uv + uuv
    }
}

impl Mul<Vector3> for Quaternion {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        let qvec = Vector3::new(self.x, self.y, self.z);
        let mut uv = qvec.cross(&rhs);
        let mut uuv = qvec.cross(&uv);
        uv *= 2.0 * self.w;
        uuv *= 2.0;
        
        rhs + uv + uuv
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl Mul<&Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Self::Output {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl Mul<Quaternion> for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl Mul<&Quaternion> for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Self::Output {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}