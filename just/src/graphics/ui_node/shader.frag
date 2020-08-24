#version 450

layout(set = 0, binding = 0) uniform texture2D colormap;
layout(set = 0, binding = 1) uniform sampler colorsampler;


layout(location = 0) in InData {
    vec2 uv;
    vec4 color;
    vec4 color_bias;

} fragment;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = (texture(sampler2D(colormap, colorsampler), fragment.uv));
    if (color.a == 0.0) {
        discard;
    }

    out_color = color;
} 
