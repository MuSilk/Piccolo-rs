#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 proj_view_matrix;
} ubo;

layout(set = 0, binding = 1) uniform UniformDynamicBufferObject {
    mat4 model;
    vec4 color;
} dynamic_ubo;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    gl_PointSize = 1.0;
    if(inColor.a == 0.0) {
        gl_Position = ubo.proj_view_matrix * dynamic_ubo.model * vec4(inPosition, 1.0);
        fragColor = dynamic_ubo.color;
    } else {
        gl_Position = vec4(inPosition, 1.0);
        fragColor = inColor;
    }
    fragTexCoord = inTexCoord;
}
/*
use texture?
use model color?
*/