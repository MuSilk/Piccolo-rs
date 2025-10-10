#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 2) uniform sampler2D texSampler;

void main() {
    if(fragTexCoord.x < 0.0 || fragTexCoord.y < 0.0) {
        outColor = fragColor;
    }
    else{
        outColor = texture(texSampler, fragTexCoord);
    }
}