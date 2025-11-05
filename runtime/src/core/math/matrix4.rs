use std::ops::{Mul, Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::core::math::vector4::Vector4;
/*
[ m[0][0]  m[1][0]  m[2][0]  m[3][0] ]   {x}
| m[0][1]  m[1][1]  m[2][1]  m[3][1] | * {y}
| m[0][2]  m[1][2]  m[2][2]  m[3][2] |   {z}
[ m[0][3]  m[1][3]  m[2][3]  m[3][3] ]   {w}
*/
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Matrix4x4 {
    m_mat: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub const fn from_columns(col1: [f32; 4], col2: [f32; 4], col3: [f32; 4], col4: [f32; 4]) -> Self {
        Matrix4x4 {
            m_mat: [col1, col2, col3, col4],
        }
    }

    pub const fn identity() -> Self {
        Matrix4x4::from_columns(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub const fn inverse(&self) -> Self {
        let m00 = self.m_mat[0][0]; 
        let m01 = self.m_mat[1][0];
        let m02 = self.m_mat[2][0];
        let m03 = self.m_mat[3][0];
        let m10 = self.m_mat[0][1];
        let m11 = self.m_mat[1][1];
        let m12 = self.m_mat[2][1];
        let m13 = self.m_mat[3][1];
        let m20 = self.m_mat[0][2]; 
        let m21 = self.m_mat[1][2];
        let m22 = self.m_mat[2][2];
        let m23 = self.m_mat[3][2];
        let m30 = self.m_mat[0][3];
        let m31 = self.m_mat[1][3];
        let m32 = self.m_mat[2][3];
        let m33 = self.m_mat[3][3];

        let v0 = m20 * m31 - m21 * m30;
        let v1 = m20 * m32 - m22 * m30;
        let v2 = m20 * m33 - m23 * m30;
        let v3 = m21 * m32 - m22 * m31;
        let v4 = m21 * m33 - m23 * m31;
        let v5 = m22 * m33 - m23 * m32;

        let t00 =   v5 * m11 - v4 * m12 + v3 * m13;
        let t10 = -(v5 * m10 - v2 * m12 + v1 * m13);
        let t20 =   v4 * m10 - v2 * m11 + v0 * m13;
        let t30 = -(v3 * m10 - v1 * m11 + v0 * m12);

        let inv_det = 1.0 / (t00 * m00 + t10 * m01 + t20 * m02 + t30 * m03);

        let d00 = t00 * inv_det;
        let d10 = t10 * inv_det;
        let d20 = t20 * inv_det;
        let d30 = t30 * inv_det;

        let d01 = -(v5 * m01 - v4 * m02 + v3 * m03) * inv_det;
        let d11 =  (v5 * m00 - v2 * m02 + v1 * m03) * inv_det;
        let d21 = -(v4 * m00 - v2 * m01 + v0 * m03) * inv_det;
        let d31 =  (v3 * m00 - v1 * m01 + v0 * m02) * inv_det;

        let v0 = m10 * m31 - m11 * m30;
        let v1 = m10 * m32 - m12 * m30;
        let v2 = m10 * m33 - m13 * m30;
        let v3 = m11 * m32 - m12 * m31;
        let v4 = m11 * m33 - m13 * m31;
        let v5 = m12 * m33 - m13 * m32;

        let d02 =  (v5 * m01 - v4 * m02 + v3 * m03) * inv_det;
        let d12 = -(v5 * m00 - v2 * m02 + v1 * m03) * inv_det;
        let d22 =  (v4 * m00 - v2 * m01 + v0 * m03) * inv_det;
        let d32 = -(v3 * m00 - v1 * m01 + v0 * m02) * inv_det;

        let v0 = m21 * m10 - m20 * m11;
        let v1 = m22 * m10 - m20 * m12;
        let v2 = m23 * m10 - m20 * m13;
        let v3 = m22 * m11 - m21 * m12;
        let v4 = m23 * m11 - m21 * m13;
        let v5 = m23 * m12 - m22 * m13;

        let d03 = -(v5 * m01 - v4 * m02 + v3 * m03) * inv_det;
        let d13 =  (v5 * m00 - v2 * m02 + v1 * m03) * inv_det;
        let d23 = -(v4 * m00 - v2 * m01 + v0 * m03) * inv_det;
        let d33 =  (v3 * m00 - v1 * m01 + v0 * m02) * inv_det;

        Matrix4x4::from_columns(
            [d00, d10, d20, d30], 
            [d01, d11, d21, d31], 
            [d02, d12, d22, d32], 
            [d03, d13, d23, d33]
        )
    }

    pub fn as_mut_ptr(&mut self) -> *mut Matrix4x4 {
        self as *mut Matrix4x4
    }
}

impl Mul for Matrix4x4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Matrix4x4 { m_mat: [[0.0; 4]; 4] };

        for i in 0..4 {
            for j in 0..4 {
                result.m_mat[i][j] = 
                    self.m_mat[0][j] * rhs.m_mat[i][0] +
                    self.m_mat[1][j] * rhs.m_mat[i][1] +
                    self.m_mat[2][j] * rhs.m_mat[i][2] +
                    self.m_mat[3][j] * rhs.m_mat[i][3];
            }
        }

        result
    }
}

impl Mul<Vector4> for Matrix4x4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Self::Output {
        Vector4 {
            x: self.m_mat[0][0] * rhs.x + self.m_mat[1][0] * rhs.y + self.m_mat[2][0] * rhs.z + self.m_mat[3][0] * rhs.w,
            y: self.m_mat[0][1] * rhs.x + self.m_mat[1][1] * rhs.y + self.m_mat[2][1] * rhs.z + self.m_mat[3][1] * rhs.w,
            z: self.m_mat[0][2] * rhs.x + self.m_mat[1][2] * rhs.y + self.m_mat[2][2] * rhs.z + self.m_mat[3][2] * rhs.w,
            w: self.m_mat[0][3] * rhs.x + self.m_mat[1][3] * rhs.y + self.m_mat[2][3] * rhs.z + self.m_mat[3][3] * rhs.w,
        }
    }
}

impl Mul<Vector4> for &Matrix4x4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Self::Output {
        Vector4 {
            x: self.m_mat[0][0] * rhs.x + self.m_mat[1][0] * rhs.y + self.m_mat[2][0] * rhs.z + self.m_mat[3][0] * rhs.w,
            y: self.m_mat[0][1] * rhs.x + self.m_mat[1][1] * rhs.y + self.m_mat[2][1] * rhs.z + self.m_mat[3][1] * rhs.w,
            z: self.m_mat[0][2] * rhs.x + self.m_mat[1][2] * rhs.y + self.m_mat[2][2] * rhs.z + self.m_mat[3][2] * rhs.w,
            w: self.m_mat[0][3] * rhs.x + self.m_mat[1][3] * rhs.y + self.m_mat[2][3] * rhs.z + self.m_mat[3][3] * rhs.w,
        }
    }
}

impl Index<usize> for Matrix4x4 {
    type Output = [f32; 4];
    fn index(&self, index: usize) -> &Self::Output {
        &self.m_mat[index]
    }
}

impl IndexMut<usize> for Matrix4x4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.m_mat[index]
    }
}

pub trait ToScaleMatrix4x4 {
    fn to_scale_matrix(&self) -> Matrix4x4;
}

impl ToScaleMatrix4x4 for f32 {
    fn to_scale_matrix(&self) -> Matrix4x4 {
        Matrix4x4::from_columns(
            [*self, 0.0, 0.0, 0.0],
            [0.0, *self, 0.0, 0.0],
            [0.0, 0.0, *self, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        )

    }
}