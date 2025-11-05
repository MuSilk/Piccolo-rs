use crate::core::math::{matrix4::Matrix4x4, vector3::Vector3};
pub fn perspective(fovy: f32, aspect: f32, z_near: f32, z_far: f32) -> Matrix4x4 {
    let tan_half_fovy = (fovy / 2.0).tan();
    Matrix4x4::from_columns(
        [1.0 / (aspect * tan_half_fovy), 0.0,                 0.0,                                  0.0],
        [0.0,                            1.0 / tan_half_fovy, 0.0,                                  0.0],
        [0.0,                            0.0,                 z_far / (z_near - z_far),            -1.0],
        [0.0,                            0.0,                 -(z_far * z_near) / (z_far - z_near), 0.0]
    )
}

pub fn look_at(eye: &Vector3, target: &Vector3, up: &Vector3) -> Matrix4x4 {
    let up = up.normalize();
    let f = (target - eye).normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f);

    Matrix4x4::from_columns(
        [s.x,         u.x,          -f.x,       0.0],
        [s.y,         u.y,          -f.y,       0.0],
        [s.z,         u.z,          -f.z,       0.0],
        [-s.dot(eye), -u.dot(eye),  f.dot(eye), 1.0]
    )
}

pub fn orthographic_projection_01(left: f32, right: f32, bottom: f32, top: f32, znear:f32, zfar: f32) -> Matrix4x4 {
    let inv_width = 1.0 / (right - left);
    let inv_height = 1.0 / (top - bottom);
    let inv_distance = 1.0 / (zfar - znear);

    let a = 2.0 * inv_width;
    let b = 2.0 * inv_height;
    let c = - (right + left) * inv_width;
    let d = - (top + bottom) * inv_height;
    let q = -1.0 * inv_distance;
    let qn = -znear * inv_distance;

    let mut proj_matrix = Matrix4x4::default();
    proj_matrix[0][0] = a;
    proj_matrix[3][0] = c;
    proj_matrix[1][1] = b;
    proj_matrix[3][1] = d;
    proj_matrix[2][2] = q;
    proj_matrix[3][2] = qn;
    proj_matrix[3][3] = 1.0;

    proj_matrix
}