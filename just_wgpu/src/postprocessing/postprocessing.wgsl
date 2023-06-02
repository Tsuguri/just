// Vertex shader

struct PostprocessingParameters {
    tint: vec4<f32>,
};

var<push_constant> pc: PostprocessingParameters;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x: f32 = -1.0 + f32(i32(in_vertex_index)%2) * 4.0;
    let y: f32 = f32(i32(in_vertex_index)/2) * 4.0 - 1.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = 0.5 * vec2<f32>(x, -y)  + vec2<f32>(0.5, 0.5);
    return out;
}


// Fragment shader

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.tex_coords);
}