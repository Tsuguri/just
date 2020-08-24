#version 450

layout(push_constant) uniform UiViewArgs {
    vec4 tex_coord_bounds;
    vec4 color;
    vec4 color_bias;
    vec2 inverse_window_size;
    vec2 coords;
    vec2 dimensions;
} push;

layout (location = 0) out gl_PerVertex {
  vec4 gl_Position;
};

layout (location = 0) out OutData {
    vec2 uv;
    vec4 color;
    vec4 color_bias;
} vert;

const vec2 positions[4] = vec2[](
    vec2(0.5, -0.5), // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5), // Right top
    vec2(-0.5, 0.5) // Left top
);

void main()
{
    vec2 pos = positions[gl_VertexIndex];
	vec2 uv= vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2)/ 2.0;

    vec2 coords_base = pos + vec2(0.5);
    vert.uv = mix(push.tex_coord_bounds.xy, push.tex_coord_bounds.zw, uv);
    vert.color = push.color;
    vert.color_bias = push.color_bias;

    vec2 center = push.coords * push.inverse_window_size;
    center.y = 1.0 - center.y; 
    vec2 final_pos = (center + push.dimensions * push.inverse_window_size * pos) * 2.0 - vec2(1.0);

    gl_Position = vec4(final_pos, 0.0, 1.0);
}
