#version 450

layout (push_constant) uniform PushConsts {
  mat4 view;
  mat4 projection;
  mat4 model;

} push;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;
layout (location = 0) out gl_PerVertex {
  vec4 gl_Position;
};
layout (location = 1) out vec4 frag_color;
void main() {
    frag_color = vec4(normal.xyz, 1.0);
    gl_Position = push.projection * push.view * push.model * vec4(pos, 1.0);
}