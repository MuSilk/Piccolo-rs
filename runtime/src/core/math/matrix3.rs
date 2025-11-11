use std::ops::{Index, IndexMut};

#[repr(C)]
pub struct Matrix3x3 {
    m_mat: [[f32; 3]; 3],
}

impl Matrix3x3 {
    pub const fn from_columns(col1: [f32; 3], col2: [f32; 3], col3: [f32; 3]) -> Self {
        Matrix3x3 {
            m_mat: [col1, col2, col3],
        }
    }
}

impl Index<usize> for Matrix3x3 {
    type Output = [f32; 3];
    fn index(&self, index: usize) -> &Self::Output {
        &self.m_mat[index]
    }
}

impl IndexMut<usize> for Matrix3x3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.m_mat[index]
    }
}
