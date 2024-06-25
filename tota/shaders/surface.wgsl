// Vertex input to the shader
struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

// Output color fragment from the shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

// Main vertex shader function
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.pos, 0., 1.);
    out.uv = in.uv;
    return out;
}

// Bindings for the texture
@group(0) @binding(0)
var t_canvas: texture_2d<f32>;

// Sampler for the texture
@group(0) @binding(1)
var s_canvas: sampler;

// Main fragment shader function
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(t_canvas, s_canvas, in.uv);
    // color = applyChromaticAberration(color, in.uv);
    // color = applyNoise(color, in.uv, -0.2);
    // color = applyBloom(color, in.uv);
    // color = applyCRT(color, in.uv); 
    // color = applyVignette(color, in.uv);
    return color;
}

fn applyGrayscale(color: vec4<f32>) -> vec4<f32> {
    let grayscale: f32 = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    return vec4<f32>(grayscale, grayscale, grayscale, color.a);
}

fn applyVignette(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let distance: f32 = length(uv - vec2<f32>(0.5, 0.5));
    let vignette: f32 = smoothstep(0.5, 0.8, distance);
    return vec4<f32>(color.rgb * (1.0 - vignette), color.a);
}

fn applyCRT(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let scanline: f32 = sin(uv.y * 800.0) * 0.1;
    let crtColor: vec3<f32> = color.rgb * (1.0 - scanline);
    return vec4<f32>(crtColor, color.a);
}

fn applyNoise(color: vec4<f32>, uv: vec2<f32>, seed: f32) -> vec4<f32> {
    let noise: f32 = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233)) + seed) * 43758.5453) * 2.0 - 1.0;
    return vec4<f32>(color.rgb + noise * 0.025, color.a);
}

fn applyBloom(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let bloomThreshold: f32 = 0.8;
    let blurRadius: f32 = 0.0015;
    var bloomColor: vec3<f32> = vec3<f32>(0.0);
    
    for (var x: i32 = -2; x <= 2; x += 1) {
        for (var y: i32 = -2; y <= 2; y += 1) {
            let offset: vec2<f32> = vec2<f32>(f32(x), f32(y)) * blurRadius;
            let sampleColor: vec4<f32> = textureSample(t_canvas, s_canvas, uv + offset);
            if (dot(sampleColor.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)) > bloomThreshold) {
                bloomColor += sampleColor.rgb;
            }
        }
    }
    
    bloomColor /= 25.0; // Normalize by the number of samples
    return vec4<f32>(color.rgb + bloomColor, color.a);
}


fn applyChromaticAberration(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let aberrationAmount: vec2<f32> = vec2<f32>(0.002, 0.002);
    let rUV: vec2<f32> = uv + aberrationAmount;
    let gUV: vec2<f32> = uv;
    let bUV: vec2<f32> = uv - aberrationAmount;
    let rColor: f32 = textureSample(t_canvas, s_canvas, rUV).r;
    let gColor: f32 = textureSample(t_canvas, s_canvas, gUV).g;
    let bColor: f32 = textureSample(t_canvas, s_canvas, bUV).b;
    return vec4<f32>(rColor, gColor, bColor, color.a);
}

fn applyEdgeDetection(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let edgeThreshold: f32 = 0.1;
    var edgeColor: vec3<f32> = vec3<f32>(0.0);

    // Define constant offsets for edge detection
    let offset0: vec2<f32> = vec2<f32>(0.0, 1.0);
    let offset1: vec2<f32> = vec2<f32>(1.0, 0.0);
    let offset2: vec2<f32> = vec2<f32>(0.0, -1.0);
    let offset3: vec2<f32> = vec2<f32>(-1.0, 0.0);

    let centerColor: vec3<f32> = textureSample(t_canvas, s_canvas, uv).rgb;

    let sampleColor0: vec3<f32> = textureSample(t_canvas, s_canvas, uv + offset0 * 0.005).rgb;
    if (length(sampleColor0 - centerColor) > edgeThreshold) {
        edgeColor = vec3<f32>(1.0);
    }

    let sampleColor1: vec3<f32> = textureSample(t_canvas, s_canvas, uv + offset1 * 0.005).rgb;
    if (length(sampleColor1 - centerColor) > edgeThreshold) {
        edgeColor = vec3<f32>(1.0);
    }

    let sampleColor2: vec3<f32> = textureSample(t_canvas, s_canvas, uv + offset2 * 0.005).rgb;
    if (length(sampleColor2 - centerColor) > edgeThreshold) {
        edgeColor = vec3<f32>(1.0);
    }

    let sampleColor3: vec3<f32> = textureSample(t_canvas, s_canvas, uv + offset3 * 0.005).rgb;
    if (length(sampleColor3 - centerColor) > edgeThreshold) {
        edgeColor = vec3<f32>(1.0);
    }

    return vec4<f32>(edgeColor, color.a);
}
