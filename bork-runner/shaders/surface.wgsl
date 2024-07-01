// Global uniform with viewport and tick fields
struct Global {
    viewport: vec2<f32>,
    tick: u32,
}

@group(0) @binding(0)
var<uniform> global: Global;

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
@group(1) @binding(0)
var t_canvas: texture_2d<f32>;

// Sampler for the texture
@group(1) @binding(1)
var s_canvas: sampler;

// Main fragment shader function
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(t_canvas, s_canvas, in.uv);
    var uv: vec2<f32> = in.uv;

    // Uncomment the following lines to apply some effects
    // color = applyRippleEffect(color, &uv, global.tick);
    // color = applyZoomPulse(color, &uv, global.tick);
    // color = applyWavy(color, &uv);
    // color = applyChromaticAberration(color, &uv);
    // color = applyBloom(color, uv);
    // color = applyCRT(color, uv);
    // color = applyVignette(color, uv);
    // color = applyColorCycle(color, uv, global.tick);
    // color = applyBlur(uv);
    // color = quantizeColor(color);
    // color = quantizeColorRetro(color);
    // color = quantizeColorPastel(color);
    // color.a -= f32(global.tick) % 0.58;

    return color;
}

fn applyBlur(uv: vec2<f32>) -> vec4<f32> {
    let offsets: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
        vec2<f32>(-1.0,  1.0), vec2<f32>( 0.0,  1.0), vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  0.0), vec2<f32>( 0.0,  0.0), vec2<f32>( 1.0,  0.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>( 0.0, -1.0), vec2<f32>( 1.0, -1.0)
    );

    let kernel: array<f32, 9> = array<f32, 9>(
        1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0,
        2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0,
        1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0
    );

    var color: vec4<f32> = vec4<f32>(0.0);

    color += textureSample(t_canvas, s_canvas, uv + offsets[0] / 256.0) * kernel[0];
    color += textureSample(t_canvas, s_canvas, uv + offsets[1] / 256.0) * kernel[1];
    color += textureSample(t_canvas, s_canvas, uv + offsets[2] / 256.0) * kernel[2];
    color += textureSample(t_canvas, s_canvas, uv + offsets[3] / 256.0) * kernel[3];
    color += textureSample(t_canvas, s_canvas, uv + offsets[4] / 256.0) * kernel[4];
    color += textureSample(t_canvas, s_canvas, uv + offsets[5] / 256.0) * kernel[5];
    color += textureSample(t_canvas, s_canvas, uv + offsets[6] / 256.0) * kernel[6];
    color += textureSample(t_canvas, s_canvas, uv + offsets[7] / 256.0) * kernel[7];
    color += textureSample(t_canvas, s_canvas, uv + offsets[8] / 256.0) * kernel[8];

    return color;
}

fn quantizeColorRetro(color: vec4<f32>) -> vec4<f32> {
    let gray: f32 = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114)); // Convert to grayscale
    if gray < 0.125 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black
    } else if gray < 0.25 {
        return vec4<f32>(0.33, 0.33, 0.33, 1.0); // Dark Gray
    } else if gray < 0.375 {
        return vec4<f32>(0.66, 0.66, 0.66, 1.0); // Light Gray
    } else if gray < 0.5 {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // White
    } else if gray < 0.625 {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    } else if gray < 0.75 {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0); // Green
    } else if gray < 0.875 {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0); // Blue
    } else {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0); // Yellow
    }
}

fn quantizeColorPastel(color: vec4<f32>) -> vec4<f32> {
    let gray: f32 = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114)); // Convert to grayscale
    if gray < 0.125 {
        return vec4<f32>(0.4, 0.2, 0.2, 1.0); // Dark pastel red
    } else if gray < 0.25 {
        return vec4<f32>(0.6, 0.4, 0.4, 1.0); // Medium pastel red
    } else if gray < 0.375 {
        return vec4<f32>(0.4, 0.4, 0.6, 1.0); // Dark pastel blue
    } else if gray < 0.5 {
        return vec4<f32>(0.6, 0.6, 0.8, 1.0); // Light pastel blue
    } else if gray < 0.625 {
        return vec4<f32>(0.4, 0.6, 0.4, 1.0); // Dark pastel green
    } else if gray < 0.75 {
        return vec4<f32>(0.6, 0.8, 0.6, 1.0); // Light pastel green
    } else if gray < 0.875 {
        return vec4<f32>(0.8, 0.8, 0.4, 1.0); // Light pastel yellow
    } else {
        return vec4<f32>(1.0, 1.0, 0.8, 1.0); // Very light pastel yellow
    }
}


fn quantizeColor(color: vec4<f32>) -> vec4<f32> {
    let gray: f32 = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114)); // Convert to grayscale
    if gray < 0.25 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Darkest color
    } else if gray < 0.5 {
        return vec4<f32>(0.33, 0.33, 0.33, 1.0); // Dark color
    } else if gray < 0.75 {
        return vec4<f32>(0.66, 0.66, 0.66, 1.0); // Light color
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // Lightest color
    }
}

fn applyRippleEffect(color: vec4<f32>, uv: ptr<function, vec2<f32>>, tick: u32) -> vec4<f32> {
    let center: vec2<f32> = vec2<f32>(0.5, 0.5);
    let distance: f32 = length(*uv - center);
    let time: f32 = f32(tick) * 0.05;
    let ripple: f32 = 0.03 * sin(10.0 * distance - time);
    *uv += normalize(*uv - center) * ripple;
    return textureSample(t_canvas, s_canvas, *uv);
}

fn applyZoomPulse(color: vec4<f32>, uv: ptr<function, vec2<f32>>, tick: u32) -> vec4<f32> {
    let center: vec2<f32> = vec2<f32>(0.5, 0.5);
    let time: f32 = f32(tick) * 0.02;
    let zoom: f32 = 1.0 + 0.1 * sin(time);
    *uv = center + (*uv - center) * zoom;
    return textureSample(t_canvas, s_canvas, *uv);
}

fn applyStrobeEffect(color: vec4<f32>, uv: vec2<f32>, tick: u32) -> vec4<f32> {
    let frequency: f32 = 10.0;
    let strobe: f32 = 0.5 + 0.5 * sin(frequency * f32(tick));
    return vec4<f32>(color.rgb * strobe, color.a);
}


fn applyTwistEffect(color: vec4<f32>, uv: ptr<function, vec2<f32>>, tick: u32) -> vec4<f32> {
    let center: vec2<f32> = vec2<f32>(0.5, 0.5);
    let time: f32 = f32(tick) * 0.01;
    let angle: f32 = distance(*uv, center) * time;
    let cosAngle: f32 = cos(angle);
    let sinAngle: f32 = sin(angle);
    let offset: vec2<f32> = *uv - center;
    *uv = vec2<f32>(
        offset.x * cosAngle - offset.y * sinAngle,
        offset.x * sinAngle + offset.y * cosAngle
    ) + center;
    return textureSample(t_canvas, s_canvas, *uv);
}

fn applyColorCycle(color: vec4<f32>, uv: vec2<f32>, tick: u32) -> vec4<f32> {
    let time: f32 = f32(tick) * 0.1;
    let r: f32 = 0.5 + 0.5 * sin(time + uv.x);
    let g: f32 = 0.5 + 0.5 * sin(time + uv.y + 2.0);
    let b: f32 = 0.5 + 0.5 * sin(time + uv.x + 4.0);
    return mix(vec4<f32>(r, g, b, color.a), color, 0.8);
}

fn applyChromaticAberration(color: vec4<f32>, uv: ptr<function, vec2<f32>>) -> vec4<f32> {
    let amount: vec2<f32> = vec2<f32>(0.002, 0.002);
    let rUV: vec2<f32> = *uv + amount;
    let gUV: vec2<f32> = *uv;
    let bUV: vec2<f32> = *uv - amount;
    let rColor: f32 = textureSample(t_canvas, s_canvas, rUV).r;
    let gColor: f32 = textureSample(t_canvas, s_canvas, gUV).g;
    let bColor: f32 = textureSample(t_canvas, s_canvas, bUV).b;
    return vec4<f32>(rColor, gColor, bColor, color.a);
}

fn applyWavy(color: vec4<f32>, uv: ptr<function, vec2<f32>>) -> vec4<f32> {
    let frequency: f32 = 20.0;
    let amplitude: f32 = 0.005;
    *uv = vec2<f32>((*uv).x + sin((*uv).y * frequency) * amplitude, (*uv).y);
    return textureSample(t_canvas, s_canvas, *uv);
}

fn applyGrayscale(color: vec4<f32>) -> vec4<f32> {
    let grayscale: f32 = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    return vec4<f32>(grayscale, grayscale, grayscale, color.a);
}

fn applyVignette(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let distance: f32 = length(uv - vec2<f32>(0.5, 0.5));
    let vignette: f32 = smoothstep(0.4, 0.7, distance);
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

fn applyColorShift(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let shiftAmount: vec2<f32> = vec2<f32>(sin(uv.y * 10.0) * 0.01, cos(uv.x * 10.0) * 0.01);
    let rUV: vec2<f32> = uv + shiftAmount;
    let gUV: vec2<f32> = uv;
    let bUV: vec2<f32> = uv - shiftAmount;
    let rColor: f32 = textureSample(t_canvas, s_canvas, rUV).r;
    let gColor: f32 = textureSample(t_canvas, s_canvas, gUV).g;
    let bColor: f32 = textureSample(t_canvas, s_canvas, bUV).b;
    return vec4<f32>(rColor, gColor, bColor, color.a);
}

fn applyRainbow(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let frequency: f32 = 10.0;
    let red: f32 = 0.5 * sin(frequency * uv.x + 0.0) + 0.5;
    let green: f32 = 0.5 * sin(frequency * uv.x + 2.0) + 0.5;
    let blue: f32 = 0.5 * sin(frequency * uv.x + 4.0) + 0.5;
    let rainbowColor: vec3<f32> = vec3<f32>(red, green, blue);
    return vec4<f32>(color.rgb * rainbowColor, color.a);
}

fn applySwirl(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let center: vec2<f32> = vec2<f32>(0.5, 0.5);
    let radius: f32 = 0.5;
    let angle: f32 = 10.0 * length(uv - center) / radius;
    let cosAngle: f32 = cos(angle);
    let sinAngle: f32 = sin(angle);
    let offsetUV: vec2<f32> = uv - center;
    let rotatedUV: vec2<f32> = vec2<f32>(
        offsetUV.x * cosAngle - offsetUV.y * sinAngle,
        offsetUV.x * sinAngle + offsetUV.y * cosAngle
    ) + center;
    return textureSample(t_canvas, s_canvas, rotatedUV);
}

fn applyGlitch(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let glitchAmount: f32 = step(0.99, fract(sin(dot(uv * 0.5, vec2<f32>(12.9898, 78.233)) * 43758.5453)));
    let glitchUV: vec2<f32> = uv + vec2<f32>(glitchAmount * 0.05, 0.0);
    return textureSample(t_canvas, s_canvas, glitchUV);
}
