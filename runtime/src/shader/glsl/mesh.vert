#version 310 es

#extension GL_GOOGLE_include_directive : enable

#include "constants.h"
#include "structures.h"

struct DirectionalLight {
    vec3  direction;
    float _padding_direction;
    vec3  color;
    float _padding_color;
};

struct PointLight {
    vec3  position;
    float radius;
    vec3  intensity;
    float _padding_intensity;
};

layout(set = 0, binding = 0) readonly buffer _unused_name_perframe {
    mat4             proj_view_matrix;
    vec3             camera_position;
    float            _padding_camera_position;
    vec3             ambient_light;
    float            _padding_ambient_light;
    uint             point_light_num;
    uint             _padding_point_light_num_1;
    uint             _padding_point_light_num_2;
    uint             _padding_point_light_num_3;
    PointLight       scene_point_lights[m_max_point_light_count];
    DirectionalLight scene_directional_light;
    highp mat4       directional_light_proj_view;
};

layout(set = 0, binding = 1) readonly buffer _unused_name_per_drawcall
{
    VulkanMeshInstance mesh_instances[m_mesh_per_drawcall_max_instance_count];
};

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_tangent;
layout(location = 3) in vec2 in_texcoord;

layout(location = 0) out vec3 out_world_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec3 out_tangent;
layout(location = 3) out vec2 out_texcoord;

void main() {
    gl_Position = proj_view_matrix * vec4(in_position, 1.0);
    out_world_position = in_position;
    out_normal = in_normal;
    out_tangent = in_tangent;
    out_texcoord = in_texcoord;
}