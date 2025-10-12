#version 310 es

layout(input_attachment_index = 0, set = 0, binding = 0) uniform highp subpassInput in_scene_color;

layout(location = 0) out highp vec4 out_color;

void main() {
    highp vec4 scene_color = subpassLoad(in_scene_color).rgba;
    out_color = scene_color;
}