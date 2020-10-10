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

layout (location = 1) out OutData {
    vec2 uv;
    vec3 worldPosition;
    vec3 normal;
} vert;


mat4 normalMat = mat4(transpose(inverse(push.model)));

void main() {
    vert.worldPosition = (push.model * vec4(pos, 1.0)).xyz;
    vert.normal = (normalMat * vec4(normal,0.0)).xyz;

    vert.uv = tex_coord;


    gl_Position = push.projection * push.view * vec4(vert.worldPosition, 1.0);
}
