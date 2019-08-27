#version 450

layout(set = 0, binding = 0) uniform texture2D colormap;
layout(set = 0, binding = 1) uniform sampler colorsampler;


layout(location = 1) in InData {
    vec2 uv;
    vec3 worldPosition;
    vec3 normal;

} fragment;

layout(location = 0) out vec4 position;
layout(location = 1) out vec4 normal;
layout(location = 2) out vec4 albedo;

void main() {

    position = vec4(fragment.worldPosition, 1.0);
    normal = vec4(fragment.normal, 1.0);
    albedo = texture(sampler2D(colormap, colorsampler), fragment.uv);
}