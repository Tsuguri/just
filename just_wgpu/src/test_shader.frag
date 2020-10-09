#version 450

layout(location=0) in vec2 in_uv;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D diffuse;
layout(set = 0, binding = 1) uniform sampler diffuse_sampler;

void main() {
    f_color = texture(sampler2D(diffuse, diffuse_sampler), in_uv);
}
