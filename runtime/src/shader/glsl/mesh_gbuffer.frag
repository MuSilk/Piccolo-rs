#version 310 es

layout(set = 1, binding = 0) uniform _unused_name_permaterial {
    highp vec4 base_color_factor;
    highp float metallic_factor;
    highp float roughness_factor;
    highp float normal_scale;
    highp float occlusion_strength;
    highp vec3 emissive_factor;
    uint is_blend;
    uint is_double_sided;
};

layout(set = 1, binding = 1) uniform sampler2D base_color_texture_sampler;
layout(set = 1, binding = 2) uniform sampler2D metallic_roughness_texture_sampler;
layout(set = 1, binding = 3) uniform sampler2D normal_texture_sampler;
layout(set = 1, binding = 4) uniform sampler2D occlusion_texture_sampler;
layout(set = 1, binding = 5) uniform sampler2D emissive_color_texture_sampler;

layout(location = 0) in highp vec3 in_world_position;
layout(location = 1) in highp vec3 in_normal;
layout(location = 2) in highp vec3 in_tangent;
layout(location = 3) in highp vec2 in_texcoord;

layout(location = 0) out highp vec4 out_scene_color;

highp vec3 get_base_color() {
    highp vec3 base_color = texture(base_color_texture_sampler, in_texcoord).xyz * base_color_factor.xyz;
    return base_color;
}

void main() {
    out_scene_color = vec4(get_base_color(), 1.0);
}