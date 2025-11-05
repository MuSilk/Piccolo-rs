

use crate::{core::math::{self, matrix4::Matrix4x4, vector3::Vector3, vector4::Vector4}, function::render::{render_camera::RenderCamera, render_scene::RenderScene}};

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

struct BoundingBox {
    min_bound: Vector3,
    max_bound: Vector3,
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
    fn merge(&mut self, point: Vector3) {
        self.min_bound.x = self.min_bound.x.min(point.x);
        self.min_bound.y = self.min_bound.y.min(point.y);
        self.min_bound.z = self.min_bound.z.min(point.z);

        self.max_bound.x = self.max_bound.x.max(point.x);
        self.max_bound.y = self.max_bound.y.max(point.y);
        self.max_bound.z = self.max_bound.z.max(point.z);
    }
    fn merge_box(&mut self, other: &BoundingBox) {
        self.min_bound.x = self.min_bound.x.min(other.min_bound.x);
        self.min_bound.y = self.min_bound.y.min(other.min_bound.y);
        self.min_bound.z = self.min_bound.z.min(other.min_bound.z);

        self.max_bound.x = self.max_bound.x.max(other.max_bound.x);
        self.max_bound.y = self.max_bound.y.max(other.max_bound.y);
        self.max_bound.z = self.max_bound.z.max(other.max_bound.z);
    }
}

fn bounding_box_transform(b: &BoundingBox, m: &Matrix4x4) -> BoundingBox { 
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

    let scene_bounding_box = {
        let mut scene_bounding_box = BoundingBox::default();

        scene.m_render_entities.iter().for_each(|(_id, entity)| {
            let mesh_asset_bounding_box = BoundingBox{
                min_bound: *entity.m_bounding_box.get_min_corner(),
                max_bound: *entity.m_bounding_box.get_max_corner(),
            };
            let mesh_bounding_box_world = bounding_box_transform(&mesh_asset_bounding_box, &entity.m_model_matrix);
            scene_bounding_box.merge_box(&mesh_bounding_box_world);
        });
        scene_bounding_box
    };

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