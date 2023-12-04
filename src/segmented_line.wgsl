// Vertex shader

const line_half_width: f32 = 40f;
const line_feathering: f32 = line_half_width * 0.5f;
// const line_feathering: f32 = 0.025;

// const dot_radius: f32 = 0.0125;

const dot_radius: f32 = line_half_width * 0f;
const dot_feathering: f32 = dot_radius * 0.5f;
// const dot_feathering: f32 = 0.00625;


struct Uniforms {
    point_to_px: mat3x2f,
    px_to_raster: mat3x2f,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> points : array<vec2f>;

struct SegmentFeathering {
    // Vertex position in pixels relative to the 1st point tangent to the segment.
    @location(0) @interpolate(linear, center) tangent_px: f32,
    // Vertex position in pixels relative to the 1st point perpendicular to the segment.
    @location(1) @interpolate(linear, center) perpendicular_px: f32,
    // Constant for a given segment, the length of the segment.
    @location(2) @interpolate(flat) segment_len_px: f32,
}

struct VertexOutput {
    @builtin(position) vertex_position: vec4<f32>,

    // --- Feathering ---
    // The fragment shader needs to know the position of the pixel it's coloring in pixel space, relative to the line segment, in order to feather the edges.
    // If we store the pixel-space position of each vertex, the fragment shader will get the linearly-interpolated position.
    // However, each vertex is used to draw _two_ segments, so we keep track of position for even and odd segments separately.

    // Even
    // Vertex position in pixels relative to the 1st point tangent to the segment.
    @location(0) @interpolate(linear, center) even_seg_tangent_px: f32,
    // Vertex position in pixels relative to the 1st point perpendicular to the segment.
    @location(1) @interpolate(linear, center) even_seg_perpendicular_px: f32,

    // Odd
    @location(2) @interpolate(linear, center) odd_seg_tangent_px: f32,
    @location(3) @interpolate(linear, center) odd_seg_perpendicular_px: f32, // TODO perp identical for both segments?

    // Constant for the current segment, the length of the segment.
    @location(4) @interpolate(flat) seg_len_px: f32,
    // Constant for the current segment, whether it's an odd segment (as opposed to even).
    @location(5) @interpolate(flat) is_odd: u32, // Bools can't be passed between vertex and fragment shaders
};

struct LineSegment {
    // Origin position in pixel space
    origin_px: vec2f,
    // Destination position in pixel space
    destination_px: vec2f,
    // Tangent unit vector in pixel space
    tan_hat_px: vec2f,
    // Length of the line segment in pixel space
    length_px: f32,
}

fn segment_from_points(origin_ind: u32, destination_ind: u32) -> LineSegment {
    var out: LineSegment;
    out.origin_px = uniforms.point_to_px * vec3f(points[origin_ind], 1f);
    out.destination_px = uniforms.point_to_px * vec3f(points[destination_ind], 1f);
    let diff = out.destination_px - out.origin_px;
    out.tan_hat_px = normalize(diff);
    out.length_px = dot(diff, out.tan_hat_px);
    return out;
}

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

    let prev_point_ind = curr_point_ind - 1u; // mod 2^32 underflow
    let next_point_ind = curr_point_ind + 1u;

    let prev_valid = curr_point_ind > 0u;
    let next_valid = next_point_ind < num_points;

    var seg0: LineSegment; // The segment preceeding this point (if one exists).
    var seg1: LineSegment; // The segment following this point (if one exists).
    var vertex_px: vec2f;
    
    // Nominal case, we're making vertices for two line segments
    if prev_valid && next_valid {
        seg0 = segment_from_points(prev_point_ind, curr_point_ind);
        seg1 = segment_from_points(curr_point_ind, next_point_ind);

        let det = (seg0.tan_hat_px.x*seg1.tan_hat_px.y) - (seg0.tan_hat_px.y*seg1.tan_hat_px.x);

        // Nominal case, lines aren't close to being parallel
        if abs(det) > 0.000000001 {
            vertex_px = seg1.origin_px + (offset * vec2f(seg1.tan_hat_px.x - seg0.tan_hat_px.x, seg1.tan_hat_px.y - seg0.tan_hat_px.y) / det);
        // Degenerate case, approaching towards divide by zero
        } else {
            // Case: Second line segment just continues in the same direction
            if dot(seg0.tan_hat_px, seg1.tan_hat_px) > 0f {
                // Whew! This we can handle correctly by just placing the vertices manually.
                vertex_px = seg1.origin_px + (offset * vec2f(-seg0.tan_hat_px.y, seg0.tan_hat_px.x));
            // Case: Second line segment reverses direction
            } else {
                // Well... shit. There isn't a great way to handle this without adding more triangles or changing the order of the vertices.
                // Best option is to just pick something big.
                vertex_px = seg1.origin_px + (offset * seg0.tan_hat_px * 1000000000f);
            }
        }

    // Beginning of line case
    } else if next_valid {
        seg1 = segment_from_points(curr_point_ind, next_point_ind);
        vertex_px = seg1.origin_px + vec2f((-offset * seg1.tan_hat_px.y) - (line_half_width * seg1.tan_hat_px.x), (offset * seg1.tan_hat_px.x) - (line_half_width * seg1.tan_hat_px.y));
    // End of line case
    } else if prev_valid {
        seg0 = segment_from_points(prev_point_ind, curr_point_ind);
        vertex_px = seg0.destination_px + vec2f((-offset * seg0.tan_hat_px.y) + (line_half_width * seg0.tan_hat_px.x), (offset * seg0.tan_hat_px.x) + (line_half_width * seg0.tan_hat_px.y));
    }

    var out: VertexOutput;

    out.vertex_position = vec4f(uniforms.px_to_raster * vec3f(vertex_px, 1f), 0f, 1f);
    out.seg_len_px = seg1.length_px;
    out.is_odd = curr_point_ind & 1u;

    var even_seg: LineSegment;
    var odd_seg: LineSegment;

    if out.is_odd == 0u {
        even_seg = seg1;
        odd_seg = seg0;
    } else {
        even_seg = seg0;
        odd_seg = seg1;
    }

    let vertex_minus_even_px = vertex_px - even_seg.origin_px;
    out.even_seg_tangent_px = dot(vertex_minus_even_px, even_seg.tan_hat_px);
    out.even_seg_perpendicular_px = dot(vertex_minus_even_px, vec2(-even_seg.tan_hat_px.y, even_seg.tan_hat_px.x));

    let vertex_minus_odd_px = vertex_px - odd_seg.origin_px;
    out.odd_seg_tangent_px = dot(vertex_minus_odd_px, odd_seg.tan_hat_px);
    out.odd_seg_perpendicular_px = dot(vertex_minus_odd_px, vec2(-odd_seg.tan_hat_px.y, odd_seg.tan_hat_px.x));

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
    // let r = f32(in.test % 5u) / 4f;
    // return vec4<f32>(in.even_seg_tangent_px / 100f, 0.0, 1.0, 0.5);
    // return vec4<f32>(in.odd_seg_tangent_px / 100f, 0.0, 1.0, 0.5);
    // return vec4<f32>((in.even_seg_perpendicular_px / 40f) + 0.5, 0.0, 1.0, 0.5);

    var seg_tangent_px: f32;
    var seg_perpendicular_px: f32;

    if in.is_odd == 0u {
        seg_tangent_px = in.even_seg_tangent_px;
        seg_perpendicular_px = in.even_seg_perpendicular_px;
    } else {
        seg_tangent_px = in.odd_seg_tangent_px;
        seg_perpendicular_px = in.odd_seg_perpendicular_px;
    }

    let outside_cap_0 = seg_tangent_px > 0f;
    let outside_cap_1 = seg_tangent_px < in.seg_len_px;

    var dist_to_line: f32;
    var temp: f32;
    if outside_cap_0 && outside_cap_1 {
        dist_to_line = abs(seg_perpendicular_px);
        temp = 0.25;
    } else {
        var tan_to_end_px = seg_tangent_px;
        temp = 0.5;
        if outside_cap_0 {
            temp = 1f;
            tan_to_end_px -= in.seg_len_px;
        }
        dist_to_line=length(vec2f(tan_to_end_px, seg_perpendicular_px));
    }

    var alpha = 1.0;
    if dist_to_line > (line_half_width - line_feathering) {
        alpha = (line_half_width - dist_to_line) / line_feathering;
    }

    // if in.is_odd == 1u {
        // return vec4<f32>(0.0, 0.0, 1.0, (0.25*alpha)+0.5);
        return vec4<f32>(0.0, 0.0, 1.0, 0.5*alpha);
    // }else {
    //     return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // }
    
    // // return vec4<f32>(0.0, 0.0, alpha, 1.0);


    // return vec4<f32>((seg_perpendicular_px - 30f)/2f, 0.3*f32(in.is_odd), 0f, 0.5*alpha);
    // return vec4<f32>(alpha, 0.3*f32(in.is_odd), 0f, 0.5*alpha);
}

@fragment
fn fs_dot(in: DotOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 0.5);
}