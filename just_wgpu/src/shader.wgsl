// Vertex shader

struct CameraUniform {
    view_projection: mat4x4<f32>,
};

struct PushConstants {
    model_matrix: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

var<push_constant> pc: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    mesh: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = mesh.tex_coords;
    out.clip_position = camera.view_projection * pc.model_matrix * vec4<f32>(mesh.position, 1.0);
    return out;
}


// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}