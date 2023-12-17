// Vertex shader

const line_half_width: f32 = 1f;
const dot_radius: f32 = 4f*line_half_width;

struct Uniforms {
    point_to_px: mat3x2f,
    px_to_raster: mat3x2f,
    color: vec4f,
    depth: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> points : array<vec2f>;

struct VertexOutput {
    @builtin(position) vertex_position: vec4<f32>,

    // --- Rounded Caps ---
    // The fragment shader needs to know the position of the pixel it's coloring in pixel space, relative to the line segment, in order to round the caps of the segments.
    // If we store the pixel-space position of each vertex, the fragment shader will get the linearly-interpolated position.
    
    // Position relative to the segment in pixel-scale coordinates.
    @location(0) @interpolate(linear, sample) segment_pos_px: vec2f,

    // Constant for the current segment, the length of the segment.
    @location(4) @interpolate(flat) segment_len_px: f32,
};

struct LineSegment {
    // Origin position in pixel space
    origin_px: vec2f,
    // Destination position in pixel space
    destination_px: vec2f,
    // Tangent unit vector in pixel space
    tan_hat_px: vec2f,
    // Perpendicular unit vector in pixel space
    perp_hat_px: vec2f,
    // Length of the line segment in pixel space
    length_px: f32,
}

fn segment_from_points(origin_ind: u32, destination_ind: u32) -> LineSegment {
    var out: LineSegment;
    out.origin_px = uniforms.point_to_px * vec3f(points[origin_ind], 1f);
    out.destination_px = uniforms.point_to_px * vec3f(points[destination_ind], 1f);
    let diff = out.destination_px - out.origin_px;
    out.tan_hat_px = normalize(diff);
    out.perp_hat_px = vec2f(-out.tan_hat_px.y, out.tan_hat_px.x);
    out.length_px = length(diff);
    return out;
}

@vertex
fn vs_line(
    @builtin(instance_index) segment_index: u32,
    // Each coord either -1 or 1
    @location(0) offset: vec2f,
) -> VertexOutput {
    var output: VertexOutput;

    let segment = segment_from_points(segment_index, segment_index + 1u);

    var position_px = segment.origin_px;
    var segment_tan_pos_px = 0f;
    if offset.x > 0f {
        position_px = segment.destination_px;
        segment_tan_pos_px = segment.length_px;
    }

    let corr_offset = (line_half_width * (mat2x2f(segment.tan_hat_px, segment.perp_hat_px)) * offset);
    // let corr_offset = (line_half_width * offset);
    output.vertex_position = vec4f(uniforms.px_to_raster * vec3f(position_px + corr_offset, 1f), uniforms.depth, 1f);
    output.segment_pos_px.x = segment_tan_pos_px + (line_half_width * offset.x);
    output.segment_pos_px.y = (line_half_width * offset.y);
    output.segment_len_px = segment.length_px;

    return output;
}

struct DotOutput {
    @builtin(position) vertex_position: vec4<f32>,
    // Position relative to the dot's center in pixel-scale coordinates.
    @location(1) @interpolate(linear, sample) dot_pos_px: vec2f,
};

@vertex
fn vs_dot(
    @builtin(instance_index) point_index: u32,
    // Each coord either -1 or 1
    @location(0) offset: vec2f,
) -> DotOutput {
    var output: DotOutput;

    let point_px = uniforms.point_to_px * vec3f(points[point_index], 1f);

    output.dot_pos_px = dot_radius * offset;
    output.vertex_position = vec4f(uniforms.px_to_raster * vec3f(point_px + output.dot_pos_px, 1f), uniforms.depth, 1f);
    
    return output;
}

// Fragment shader
@fragment
fn fs_line(in: VertexOutput) -> @location(0) vec4<f32> {
    let outside_cap_0 = in.segment_pos_px.x > 0f;
    let outside_cap_1 = in.segment_pos_px.x < in.segment_len_px;

    if in.segment_pos_px.x < 0f {
        if length(in.segment_pos_px) > line_half_width {
            discard;
        }
    } else if in.segment_pos_px.x > in.segment_len_px {
        if length(vec2f(in.segment_pos_px.x - in.segment_len_px, in.segment_pos_px.y)) > line_half_width {
            discard;
        }
    }

    // let alpha = 0.5f;
    let alpha = 0.8f;

    return uniforms.color;
}

@fragment
fn fs_dot(in: DotOutput) -> @location(0) vec4<f32> {
    if length(in.dot_pos_px) > dot_radius {
        discard;
    }

    let alpha = 0.8f;

    return uniforms.color;
}