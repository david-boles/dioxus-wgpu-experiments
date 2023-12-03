// Vertex shader

const line_half_width: f32 = 20f;
const line_feathering: f32 = line_half_width * 0.5f;
// const line_feathering: f32 = 0.025;

// const dot_radius: f32 = 0.0125;

const dot_radius: f32 = line_half_width * 0.5f;
const dot_feathering: f32 = dot_radius * 0.5f;
// const dot_feathering: f32 = 0.00625;


struct Uniforms {
    point_to_px: mat3x2f,
    px_to_raster: mat3x2f,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> points : array<vec2f>;

struct VertexOutput {
    @builtin(position) vertex_position: vec4<f32>,
};

@vertex
fn vs_line(
    @builtin(vertex_index) vertex_index : u32,
) -> VertexOutput {
    let num_points: u32 = arrayLength(&points);

    // Odd vertices will be "below" the line, evens above
    var offset = line_half_width;
    if (vertex_index & 1u) == 0u {
        offset = -line_half_width;
    }

    let curr_point_ind = vertex_index >> 1u;
    let curr_point = uniforms.point_to_px * vec3f(points[curr_point_ind], 1f);

    let prev_point_ind = curr_point_ind - 1u; // mod 2^32 underflow
    let next_point_ind = curr_point_ind + 1u;

    let prev_valid = curr_point_ind > 0u;
    let next_valid = next_point_ind < num_points;

    var vertex_pos: vec2f;

    // Nominal case, we're making vertices for two line segments
    if prev_valid && next_valid {
        var prev_point = uniforms.point_to_px * vec3f(points[prev_point_ind], 1f);
        var next_point = uniforms.point_to_px * vec3f(points[next_point_ind], 1f);
        // prev_point = uniforms.px_to_raster * vec3f(prev_point, 1f);
        // next_point = uniforms.px_to_raster * vec3f(next_point, 1f);

        let l1_hat = normalize(curr_point - prev_point);
        let l2_hat = normalize(next_point - curr_point);

        let det = (l1_hat.x*l2_hat.y) - (l1_hat.y*l2_hat.x);

        // Nominal case, lines aren't close to being parallel
        if abs(det) > 0.000000001 {
            vertex_pos = curr_point + (offset * vec2(l2_hat.x - l1_hat.x, l2_hat.y - l1_hat.y) / det);
        // Degenerate case, approaching towards divide by zero
        } else {
            // Case: Line segment just continues in the same direction
            if dot(l1_hat, l2_hat) > 0f {
                // Whew! This we can handle correctly by just placing the vertices manually.
                vertex_pos = curr_point + (offset * vec2(-l1_hat.y, l1_hat.x));
            // Case: line segment reverses direction
            } else {
                // Well... shit. There isn't a great way to handle this without adding more triangles or changing the order of the vertices.
                // Best option is to just pick something big.
                vertex_pos = curr_point + (offset * l1_hat * 1000000000f);
            }
        }

    // Beginning of line case
    } else if next_valid {
        var next_point = uniforms.point_to_px * vec3f(points[next_point_ind], 1f);
        // next_point = uniforms.px_to_raster * vec3f(next_point, 1f);

        let l_hat = normalize(next_point - curr_point);
        vertex_pos = curr_point + vec2((-offset * l_hat.y) - (line_half_width * l_hat.x), (offset * l_hat.x) - (line_half_width * l_hat.y));
    // End of line case
    } else if prev_valid {
        var prev_point = uniforms.point_to_px * vec3f(points[prev_point_ind], 1f);
        // prev_point = uniforms.px_to_raster * vec3f(prev_point, 1f);

        let l_hat = normalize(curr_point - prev_point);
        vertex_pos = curr_point + vec2((-offset * l_hat.y) + (line_half_width * l_hat.x), (offset * l_hat.x) + (line_half_width * l_hat.y));
    }

    // var foo: vec4<f32> = vec4

    var out: VertexOutput;
    // out.vertex_position = vec4(vertex_pos, 0f, 1f);
    out.vertex_position = vec4(uniforms.px_to_raster * vec3f(vertex_pos, 1f), 0f, 1f);
    return out;
}

struct DotOutput {
    @builtin(position) vertex_position: vec4<f32>,
};

// TODO naga/wgpu bug prevents this from being a const
var<private> dot_offset_lut: array<vec2f, 6> = array<vec2f, 6>(
        // First triangle
        vec2(-1f, -1f),
        vec2(-1f, 1f),
        vec2(1f, -1f),
        // Second triangle
        vec2(1f, 1f),
        vec2(1f, -1f),
        vec2(-1f, 1f),
    );

@vertex
fn vs_dot(@builtin(vertex_index) vertex_index : u32) -> DotOutput {
    let lut_index: u32 = vertex_index % 6u;
    let point = uniforms.point_to_px * vec3f(points[vertex_index / 6u], 1f);

    let vertex = point + (dot_offset_lut[lut_index] * vec2(dot_radius, dot_radius));

    var out: DotOutput;
    out.vertex_position = vec4(uniforms.px_to_raster * vec3f(vertex, 1f), 0f, 1f);
    
    return out;
}

// Fragment shader

@fragment
fn fs_line(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 0.5);
}

@fragment
fn fs_dot(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 0.5);
}