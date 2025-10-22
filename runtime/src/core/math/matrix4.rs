use std::ops::{Mul, Index, IndexMut};

use crate::core::math::vector4::Vector4;
/*
[ m[0][0]  m[1][0]  m[2][0]  m[3][0] ]   {x}
| m[0][1]  m[1][1]  m[2][1]  m[3][1] | * {y}
| m[0][2]  m[1][2]  m[2][2]  m[3][2] |   {z}
[ m[0][3]  m[1][3]  m[2][3]  m[3][3] ]   {w}
*/
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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