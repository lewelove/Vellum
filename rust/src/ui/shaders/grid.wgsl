struct Globals {
    viewport_size: vec2<f32>,
    scroll_y: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;

@group(1) @binding(0) var t_diffuse: texture_2d_array<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct AlbumInstance {
    @location(0) position: vec2<f32>,
    @location(1) tex_index: i32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_idx: i32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    instance: AlbumInstance,
) -> VertexOutput {
    let quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    let size = vec2<f32>(190.0, 190.0);
    let pos = instance.position + quad_pos[v_idx] * size;
    let scrolled_pos = vec2<f32>(pos.x, pos.y - globals.scroll_y);
    
    let safe_viewport = max(globals.viewport_size, vec2<f32>(1.0, 1.0));
    let ndc_pos = (scrolled_pos / safe_viewport) * 2.0 - 1.0;
    
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(ndc_pos.x, -ndc_pos.y, 0.0, 1.0);
    out.uv = quad_pos[v_idx];
    out.tex_idx = instance.tex_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.tex_idx >= 0) {
        return textureSample(t_diffuse, s_diffuse, in.uv, in.tex_idx);
    }
    return vec4<f32>(0.2, 0.2, 0.2, 1.0);
}
