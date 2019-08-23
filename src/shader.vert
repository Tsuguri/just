#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;
layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = vec4(normal, 1.0);
    gl_Position = vec4(pos, 1.0);
}