#version 310 es

layout(input_attachment_index = 0, set = 0, binding = 0) uniform highp subpassInput in_scene_color;

layout(input_attachment_index = 1, set = 0, binding = 1) uniform highp subpassInput in_ui_color;

layout(location = 0) out highp vec4 out_color;

void main() {
    highp vec4 scene_color = subpassLoad(in_scene_color).rgba;
    highp vec4 ui_color = subpassLoad(in_ui_color).rgba;
    out_color = ui_color.a * ui_color + (1.0 - ui_color.a) * scene_color;
}