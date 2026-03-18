enable dual_source_blending;

struct Globals {
    viewport_size: vec2<f32>,
    scroll_y: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) @blend_src(1) mask: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.uv = vec2<f32>(0.0, 0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.mask = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}
