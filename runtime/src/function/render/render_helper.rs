

use crate::{core::math::{self, bounding_box::{BoundingBox, bounding_box_transform}, matrix4::Matrix4x4, vector3::Vector3, vector4::Vector4}, function::render::{render_camera::RenderCamera, render_scene::RenderScene}};

#[inline]
pub fn round_up(value: u32, alignment: u32) -> u32 {
    let temp = value + alignment -1;
    return temp - temp % alignment;
}

struct ClusterFrustum {
    m_plane_right: Vector4,
    m_plane_left: Vector4,
    m_plane_top: Vector4,
    m_plane_bottom: Vector4,
    m_plane_near: Vector4,
    m_plane_far: Vector4,
}

pub fn calculate_directional_light_camera(scene: &RenderScene, camera: &RenderCamera) -> Matrix4x4 {
    let proj_view_matrix = {
        camera.get_perspective_matrix() * camera.get_view_matrix()
    };

    let frustum_bounding_box = {
        let g_frustum_points_ndc_space = [
            Vector3::new(-1.0, -1.0, 1.0),
            Vector3::new( 1.0, -1.0, 1.0),
            Vector3::new( 1.0,  1.0, 1.0),
            Vector3::new(-1.0,  1.0, 1.0),
            Vector3::new(-1.0, -1.0, 0.0),
            Vector3::new( 1.0, -1.0, 0.0),
            Vector3::new( 1.0,  1.0, 0.0),
            Vector3::new(-1.0,  1.0, 0.0),
        ];
        let inverse_proj_view_matrix = proj_view_matrix.inverse();

        let mut frustum_bounding_box = BoundingBox::default();

        for i in 0..8 {
            let frustum_point_with_w = inverse_proj_view_matrix * g_frustum_points_ndc_space[i].to_homogeneous();
            let frustum_point = Vector3::from_homogeneous(&frustum_point_with_w);
            frustum_bounding_box.merge(frustum_point);
        }
        frustum_bounding_box
    };

    let scene_bounding_box = scene.calc_scene_bounding_box();

    let box_center = (frustum_bounding_box.min_bound + frustum_bounding_box.max_bound) *0.5;
    let box_extent = (frustum_bounding_box.max_bound - frustum_bounding_box.min_bound) *0.5;
    let eye = box_center + scene.m_directional_light.m_direction * box_extent.length();
    let center = box_center;
    let light_view = math::look_at(&eye, &center,&Vector3::new(0.0, 0.0, 1.0));

    let frustum_bounding_box_light_view = bounding_box_transform(&frustum_bounding_box, &light_view);
    let scene_bounding_box_light_view = bounding_box_transform(&scene_bounding_box, &light_view);
    let light_proj = math::orthographic_projection_01(
        frustum_bounding_box_light_view.min_bound.x.max(scene_bounding_box_light_view.min_bound.x),
        frustum_bounding_box_light_view.max_bound.x.min(scene_bounding_box_light_view.max_bound.x),
        frustum_bounding_box_light_view.min_bound.y.max(scene_bounding_box_light_view.min_bound.y),
        frustum_bounding_box_light_view.max_bound.y.min(scene_bounding_box_light_view.max_bound.y),
        -scene_bounding_box_light_view.max_bound.z,
        -(frustum_bounding_box_light_view.min_bound.z.max(scene_bounding_box_light_view.min_bound.z))
    );
    light_proj * light_view
}