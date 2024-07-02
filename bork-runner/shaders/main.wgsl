struct Global {
    viewport: vec2<f32>,
    tick: u32,
}

@group(0) @binding(0)
var<uniform> global: Global;

//------------------------------------------------------------------------------
// Vertex shader
//------------------------------------------------------------------------------

struct VertexInput {
    @location(0) rect: vec4<f32>,
    @location(1) fill: u32,
    @location(2) tex_rect: vec4<f32>,
    @location(3) tex_fill: u32,
    @location(4) rotation_base: f32,
    @location(5) rotation_rate: f32,
    @location(6) transform_origin: vec2<f32>,
    @location(7) border_radius: vec2<u32>,
    @location(8) border_size: u32,
    @location(9) border_color: vec4<u32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) bg_fill: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tex_area: f32,
    @location(3) tex_fill: vec4<f32>,
    @location(4) rect: vec4<f32>,
    @location(5) angle: f32,
    @location(6) rot_origin: vec2<f32>,
    @location(7) border_size: vec4<f32>,
    @location(8) border_top_left_radius: vec2<f32>,
    @location(9) border_top_right_radius: vec2<f32>,
    @location(10) border_bottom_right_radius: vec2<f32>,
    @location(11) border_bottom_left_radius: vec2<f32>,    
    @location(12) border_color_t: vec4<f32>,
    @location(13) border_color_b: vec4<f32>,
    @location(14) border_color_l: vec4<f32>,
    @location(15) border_color_r: vec4<f32>,
}


@vertex
fn vs_main(in: VertexInput, @builtin(vertex_index) i: u32) -> VertexOutput {
    // let i = n % 6u;
    // let i = in.index;
    // let i = n;
    let tick = f32(global.tick);
    // let viewport_size = floor(global.viewport / global.scale);
    let viewport_pos = vec2<f32>(0., 0.);
    let viewport_size = global.viewport;

    // Calc vertex output coords
    var vec_pos: vec2<f32>;
    var tex_pos: vec2<f32>;

    // let vx0 = in.rect.x;
    let vx0 = in.rect.x + viewport_pos.x;
    let vy0 = in.rect.y + viewport_pos.y;
    let vx1 = in.rect.z + vx0;
    let vy1 = in.rect.w + vy0;

    var tx0 = in.tex_rect.x;
    var ty0 = in.tex_rect.y;
    var tx1 = in.tex_rect.z + tx0;
    // flip x
    if tx1 < tx0 {
        tx1 = tx0;
        tx0 = abs(in.tex_rect.z) + tx0;
    }
    var ty1 = in.tex_rect.w + ty0;
    // flip y
    if ty1 < ty0 {
        ty1 = ty0;
        ty0 = abs(in.tex_rect.w) + ty0;
    }

    // bottom-right
    if i == 0u || i == 5u {
        vec_pos.x = vx1;
        vec_pos.y = vy1;
        tex_pos.x = tx1;
        tex_pos.y = ty1;
    }
    // top-right
    if i == 1u {
        vec_pos.x = vx1;
        vec_pos.y = vy0;
        tex_pos.x = tx1;
        tex_pos.y = ty0;
    }
    // top-left
    if i == 2u || i == 3u {
        vec_pos.x = vx0;
        vec_pos.y = vy0;
        tex_pos.x = tx0;
        tex_pos.y = ty0;
    }
    // bottom-left
    if i == 4u {
        vec_pos.x = vx0;
        vec_pos.y = vy1;
        tex_pos.x = tx0;
        tex_pos.y = ty1;
    }
    vec_pos = floor(vec_pos);
    tex_pos = floor(tex_pos);

    // border-radius can't be larger than 50% of the rect w/h
    let brx = to_vec4_f32(in.border_radius.x).x;
    let bry = to_vec4_f32(in.border_radius.y).x;
    var br = vec2<f32>(
        min(brx, in.rect.z / 2.),
        min(bry, in.rect.w / 2.),
        // min(in.border_radius.x, in.rect.z / 2.),
        // min(in.border_radius.y, in.rect.w / 2.),
    );

    // Apply rotation
    let angle = in.rotation_base + (in.rotation_rate * tick);
    var pos = vec2<f32>(
        vx0 + in.transform_origin.x,
        vy0 + in.transform_origin.y
    );
    let size = in.rect.zw;
    var rot_origin = pos + size / 2.0;
    if angle != 0. {
        let rot_mat = mat2x2<f32>(
            cos(angle),
            -sin(angle),
            sin(angle),
            cos(angle)
        );
        vec_pos -= rot_origin;
        vec_pos *= rot_mat;
        vec_pos += rot_origin;
    }
    // Convert xy to normalized device coordinates (NDC)
    vec_pos = (vec_pos / viewport_size) * 2. - 1.;

    var out: VertexOutput;
    out.pos = vec4<f32>(vec_pos * vec2<f32>(1., -1.), 0., 1.);
    out.tex_coords = tex_pos;
    // out.bg_fill = quantize_color(to_rgba(in.fill));
    out.bg_fill = to_rgba(in.fill);
    out.tex_fill = to_rgba(in.tex_fill);
    // out.tex_area = abs(in.tex_rect.z) * abs(in.tex_rect.w);
    out.border_top_left_radius = vec2<f32>(
        min(to_vec4_f32(in.border_radius.x).x, in.rect.z / 2.),
        min(to_vec4_f32(in.border_radius.y).x, in.rect.w / 2.),
    );
    out.border_top_right_radius = vec2<f32>(
        min(to_vec4_f32(in.border_radius.x).y, in.rect.z / 2.),
        min(to_vec4_f32(in.border_radius.y).y, in.rect.w / 2.),
    );
    out.border_bottom_right_radius = vec2<f32>(
        min(to_vec4_f32(in.border_radius.x).z, in.rect.z / 2.),
        min(to_vec4_f32(in.border_radius.y).z, in.rect.w / 2.),
    );
    out.border_bottom_left_radius = vec2<f32>(
        min(to_vec4_f32(in.border_radius.x).w, in.rect.z / 2.),
        min(to_vec4_f32(in.border_radius.y).w, in.rect.w / 2.),
    );
    out.rect = in.rect;
    out.angle = angle;
    out.rot_origin = rot_origin;
    out.border_size = to_vec4_f32(in.border_size);
    out.border_color_t = to_rgba(in.border_color.x);
    out.border_color_r = to_rgba(in.border_color.y);
    out.border_color_b = to_rgba(in.border_color.z);
    out.border_color_l = to_rgba(in.border_color.w);
    return out;
}

// https://github.com/mvlabat/bevy_egui/blob/main/src/egui.wgsl
fn linear_from_srgb(srgb: vec4<f32>) -> vec4<f32> {
    let cutoff = srgb.rgb < vec3<f32>(0.04045);
    let lower = srgb.rgb / 12.92;
    let higher = pow((srgb.rgb + 0.055) / 1.055, vec3<f32>(2.4));
    let out = select(higher, lower, cutoff);
    return vec4<f32>(out.rgb, srgb.a);
}


fn to_rgba(color: u32) -> vec4<f32> {
    let r = to_linear_component(color);
    let g = to_linear_component(color >> 8u);
    let b = to_linear_component(color >> 16u);
    let a = to_linear_component(color >> 24u);
    let linear = vec4<f32>(r, g, b, a);
    return linear;
}

// Converts linear RGB to sRGB
// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_sRGB_decode.txt
// https://github.com/three-rs/three/blob/07e47da5e0673aa9a16526719e16debd59040eec/src/color.rs#L39
fn to_linear_component(xu: u32) -> f32 {
    let x = f32(xu & 0xFFu) / 255.0;
    if x > 0.04045 {
        return pow((x + 0.055) / 1.055, 2.4);
    }
    return x / 12.92;
}

fn to_srgb(color: vec4<f32>) -> vec4<f32> {
    let r = pow(color.r, 2.2);
    let g = pow(color.g, 2.2);
    let b = pow(color.b, 2.2);
    let a = pow(color.a, 2.2);
    return vec4<f32>(r, g, b, a);
}

fn to_vec4_f32(color: u32) -> vec4<f32> {
    let x = f32(((color >> 0u) & 0xFFu));
    let y = f32(((color >> 8u) & 0xFFu));
    let z = f32(((color >> 16u) & 0xFFu));
    let w = f32(((color >> 24u) & 0xFFu));
    return vec4<f32>(x, y, z, w);
}

const palette: array<vec3<f32>, 4> = array<vec3<f32>, 4>(
    vec3<f32>(1.0, 0.0, 0.0), // Red
    vec3<f32>(0.0, 1.0, 0.0), // Green
    vec3<f32>(0.0, 0.0, 1.0), // Blue
    vec3<f32>(1.0, 1.0, 0.0), // Yellow
);
 

//------------------------------------------------------------------------------
// Fragment shader
//------------------------------------------------------------------------------

@group(1) @binding(0)
var t_spritesheet: texture_2d<f32>;

@group(1) @binding(1)
var s_spritesheet: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let viewport_pos = vec2<f32>(0., 0.);
    let tex_dims = textureDimensions(t_spritesheet);
    let tex_dimsf = vec2<f32>(tex_dims.xy);
    let uv = in.tex_coords.xy / tex_dimsf;
    // Default sample is top-left pixel, which should be transparent
    var tex_color = textureSample(t_spritesheet, s_spritesheet, uv);
    // Apparently, only the alpha sample is sRGB
    tex_color.a = to_linear_component(u32(tex_color.a * 255.));
    // tex_color = in.color * vec4<f32>(texture_color.rgb * texture_color.a, texture_color.a);
    // tex_color = quantize_color(tex_color);
    
    // Unapply transformation so we can do calculations easier
    let rot_mat = mat2x2<f32>(
        cos(in.angle),
        sin(in.angle),
        -sin(in.angle),
        cos(in.angle)
    );
    var pos = in.pos.xy;
    let rot_origin = in.rot_origin;
    pos -= rot_origin;
    pos *= rot_mat;
    pos += rot_origin;

    let px = pos.x;
    let py = pos.y;
    let rx = in.rect.x + viewport_pos.x;
    let ry = in.rect.y + viewport_pos.y;
    let rw = in.rect.z;
    let rh = in.rect.w;

    let bst = in.border_size.x; // top
    let bsr = in.border_size.y; // right
    let bsb = in.border_size.z; // bottom
    let bsl = in.border_size.w; // left

    // Border radius (top-left)
    let brtl = in.border_top_left_radius;
    if brtl.x > 0. || brtl.y > 0. {
        var e: vec4<f32>;
        e.z = brtl.x * 2.;
        e.w = brtl.y * 2.;
        e.x = rx;
        e.y = ry;
        var r: vec4<f32>;
        r.z = e.z * .5;
        r.w = e.w * .5;
        r.x = e.x;
        r.y = e.y;
        if intersects_rect(px, py, r.x, r.y, r.z, r.w) {
            if !intersects_ellipse(px, py, e.x, e.y, e.z, e.w) {
                    discard;
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bst { // top
                if intersects_rect(px, py, r.x, r.y, r.z, r.w * .5) {
                    return in.border_color_t;
                }
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bsl { // left
                if intersects_rect(px, py, r.x, r.y, r.z * .5, r.w) {
                    return in.border_color_l;
                }
            }
        }
    }
        
    // Border radius (top-right)
    let brtr = in.border_top_right_radius;
    if brtr.x > 0. || brtr.y > 0. {
        var e: vec4<f32>;
        e.z = brtr.x * 2.;
        e.w = brtr.y * 2.;
        e.x = rx + rw + -e.z;
        e.y = ry;
        var r: vec4<f32>;
        r.z = e.z * .5;
        r.w = e.w * .5;
        r.x = e.x + r.z;
        r.y = e.y;
        if intersects_rect(px, py, r.x, r.y, r.z, r.w) {
            if !intersects_ellipse(px, py, e.x, e.y, e.z, e.w) {
                    discard;
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bst { // top
                if intersects_rect(px, py, r.x, r.y, r.z * .5, r.w) {
                    return in.border_color_t;
                }
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bsr { // right
                if intersects_rect(px, py, r.x + r.z * .5, r.y, r.z * .5, r.w) {
                    return in.border_color_r;
                }
            }
        }
    }

    // Border radius (bottom-right)
    let brbr = in.border_bottom_right_radius;
    if brbr.x > 0. || brbr.y > 0. {
        var e: vec4<f32>;
        e.z = brbr.x * 2.;
        e.w = brbr.y * 2.;
        e.x = rx + rw + -e.z;
        e.y = ry + rh + -e.w;
        var r: vec4<f32>;
        r.z = e.z * .5;
        r.w = e.w * .5;
        r.x = e.x + r.z;
        r.y = e.y + r.w;
        if intersects_rect(px, py, r.x, r.y, r.z, r.w) {
            if !intersects_ellipse(px, py, e.x, e.y, e.z, e.w) {
                    discard;
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bst { // bottom
                if intersects_rect(px, py, r.x, r.y + r.w * .5, r.z, r.w * .5) {
                    return in.border_color_b;
                }
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bsr { // right
                if intersects_rect(px, py, r.x + r.z * .5, r.y, r.z * .5, r.w) {
                    return in.border_color_r;
                }
            }
        }
    }

    // Border radius (bottom-left)
    let brbl = in.border_bottom_left_radius;
    if brbl.x > 0. || brbl.y > 0. {
        var e: vec4<f32>;
        e.z = brbl.x * 2.;
        e.w = brbl.y * 2.;
        e.x = rx;
        e.y = ry + rh + -e.w;
        var r: vec4<f32>;
        r.z = e.z * .5;
        r.w = e.w * .5;
        r.x = e.x;
        r.y = e.y + r.w;
        if intersects_rect(px, py, r.x, r.y, r.z, r.w) {
            if !intersects_ellipse(px, py, e.x, e.y, e.z, e.w) {
                    discard;
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bst { // bottom
                if intersects_rect(px, py, r.x, r.y + r.w * .5, r.z, r.w * .5) {
                    return in.border_color_b;
                }
            }
            if ellipse_edge_distance(px, py, e.x, e.y, e.z, e.w) <= bsl { // left
                if intersects_rect(px, py, r.x, r.y, r.z * .5, r.w) {
                    return in.border_color_l;
                }
            }
        }
    }

    // Border color (top)
    if intersects_rect(px, py, rx, ry, rw, bst) {
        return in.border_color_t;
    }

    // Border color (right)
    if intersects_rect(px, py, (rx + rw) - bsr, ry, bsr, rh) {
        return in.border_color_r;
    }

    // Border color (bottom)
    if intersects_rect(px, py, rx, (ry + rh) - bsb, rw, bsb) {
        return in.border_color_b;
    }

    // Border color (left)
    if intersects_rect(px, py, rx, ry, bsl, rh) {
        return in.border_color_l;
    }
    
    // bg color
    if in.bg_fill.a > 0. && tex_color.a == 0. {
        return in.bg_fill;
    }

    // Sampled texture color
    return tex_color * in.tex_fill;
}

// The equation of an ellipse centered at the point (h, k) with semi-major axis 'a' and semi-minor axis 'b' is:
// ((x - h)^2 / a^2) + ((y - k)^2 / b^2) <= 1
fn intersects_ellipse(pointX: f32, pointY: f32, topLeftX: f32, topLeftY: f32, width: f32, height: f32) -> bool {
    let radiusX = width / 2.;
    let radiusY = height / 2.;

    let centerX = topLeftX + radiusX;
    let centerY = topLeftY + radiusY;

    let distanceX = pointX - centerX;
    let distanceY = pointY - centerY;

    return ((distanceX * distanceX) / (radiusX * radiusX) + (distanceY * distanceY) / (radiusY * radiusY)) < 1.;
}

fn ellipse_edge_distance(pointX: f32, pointY: f32, topLeftX: f32, topLeftY: f32, width: f32, height: f32) -> f32 {
    let radiusX = width / 2.;
    let radiusY = height / 2.;

    let centerX = topLeftX + radiusX;
    let centerY = topLeftY + radiusY;

    var dx = pointX - centerX;
    var dy = pointY - centerY;

    var angle = atan2(dy * radiusX, dx * radiusY);
    var ellipseX = radiusX * cos(angle);
    var ellipseY = radiusY * sin(angle);

    var distanceX = abs(ellipseX - dx);
    var distanceY = abs(ellipseY - dy);

    var distance = sqrt(distanceX * distanceX + distanceY * distanceY);
    return distance;
}

fn intersects_rect(pointX: f32, pointY: f32, rectX: f32, rectY: f32, rectWidth: f32, rectHeight: f32) -> bool {
    return (pointX >= rectX && // Point is to the right of the left edge
    pointX <= rectX + rectWidth && // Point is to the left of the right edge
    pointY >= rectY && // Point is below the top edge
    pointY <= rectY + rectHeight); // Point is above the bottom edge
}
