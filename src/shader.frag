#version 450


layout(location = 1) in vec4 frag_color;
layout(location = 0) out vec4 color;

void main() {
    color = frag_color;
}