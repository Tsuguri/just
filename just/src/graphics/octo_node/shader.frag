#version 450

layout(set = 0, binding = 0) uniform sampler colorsampler;
layout(set = 0, binding = 1) uniform texture2D[3] colormaps;

layout(location = 1) in vec2 uv;
layout(location = 0) out vec4 color;

void main() {
    vec4 pos = texture(sampler2D(colormaps[0], colorsampler), uv);
    vec4 normal = texture(sampler2D(colormaps[1], colorsampler), uv);
    vec4 albedo = texture(sampler2D(colormaps[2], colorsampler), uv);
    color = (pos + normal + albedo) / 3.0;
    color.w = 1.0;
}