// Vertex shader

const width: f32 = 0.1;

@group(0) @binding(0) var<storage, read> points : array<vec2<f32>>;

struct VertexOutput {
    @builtin(position) vertex_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index : u32,
) -> VertexOutput {
    // var position = points[vertex_index >> u32(1)];
    var position = points[vertex_index / u32(2)];

    if (vertex_index & u32(1)) == u32(0) {
        position[1] += width;
    } else {
        position[1] -= width;
    }

    var out: VertexOutput;
    out.vertex_position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}