#version 450

layout(set = 0, binding = 0) uniform texture2D colormap;
layout(set = 0, binding = 1) uniform sampler colorsampler;

layout(location = 1) in vec2 uv;
layout(location = 0) out vec4 color;

void main() {
    color = texture(sampler2D(colormap, colorsampler), uv);
    //if(uv.x>0.5){
    //color = vec4(uv, 0.0, 1.0);}else
    //{color = vec4(0.0, 1.0, 0.0,1.0);}
}