#version 310 es

layout(location = 0) in highp vec3 in_world_position;
layout(location = 1) in highp vec3 in_normal;
layout(location = 2) in highp vec3 in_tangent;
layout(location = 3) in highp vec2 in_texcoord;

layout(location = 0) out highp vec4 out_scene_color;

void main() {
    out_scene_color = vec4(0.0, 0.0, 0.0 , 1.0);
}