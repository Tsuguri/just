#version 450

layout(location = 1) in InData {
    vec2 uv;
    vec3 worldPosition;
    vec3 normal;

} fragment;

layout(location=0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D diffuse;
layout(set = 0, binding = 1) uniform sampler diffuse_sampler;

void main() {
    color = texture(sampler2D(diffuse, diffuse_sampler), fragment.uv);
    //color = vec4(fragment.normal, 1.0);
}
