enable dual_source_blending;

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
    @location(2) is_text: i32,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<f32>,
    @location(2) @interpolate(flat) tex_idx: i32,
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

    var size = vec2<f32>(190.0, 190.0);
    if (instance.is_text == 1) {
        size = vec2<f32>(190.0, 32.0);
    }

    // Expand the quad by 1 pixel in all directions so the rasterizer 
    // covers the fragments immediately outside the mathematical boundary.
    let expand_dir = quad_pos[v_idx] * 2.0 - 1.0; // Pushes out to [-1, 1]
    let expanded_pos = instance.position + quad_pos[v_idx] * size + expand_dir * 1.0;
    
    let scrolled_pos = vec2<f32>(expanded_pos.x, expanded_pos.y - globals.scroll_y);
    
    let safe_viewport = max(globals.viewport_size, vec2<f32>(1.0, 1.0));
    let ndc_pos = (scrolled_pos / safe_viewport) * 2.0 - 1.0;
    
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(ndc_pos.x, -ndc_pos.y, 0.0, 1.0);
    out.local_pos = quad_pos[v_idx] * size + expand_dir * 1.0;
    out.size = size;
    out.tex_idx = instance.tex_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate precise distance to the mathematical edge in pixels
    let dist_x = min(in.local_pos.x, in.size.x - in.local_pos.x);
    let dist_y = min(in.local_pos.y, in.size.y - in.local_pos.y);
    let edge_dist = min(dist_x, dist_y);

    // Fade the alpha identically to MSAA coverage over a 1-pixel boundary
    let alpha = clamp(edge_dist + 0.5, 0.0, 1.0);
    
    // UV is calculated against the original unexpanded size
    let uv = in.local_pos / in.size;

    if (in.tex_idx >= 0) {
        let color = textureSample(t_diffuse, s_diffuse, uv, in.tex_idx);
        return vec4<f32>(color.rgb, color.a * alpha);
    }
    
    return vec4<f32>(0.1, 0.1, 0.1, alpha);
}
