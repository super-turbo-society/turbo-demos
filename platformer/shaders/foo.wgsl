// Global uniform with viewport and tick fields
struct Global {
    camera: vec3<f32>,
    tick: u32,
    viewport: vec2<f32>,
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
    return textureSample(t_canvas, s_canvas, in.uv);
    // if global.camera.z == 1. {
    // } else {
        // return quantizedTextureSample(t_canvas, s_canvas, in.uv);
    // }
}

fn quantizedTextureSample(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec4<f32> {
    // Zoom factor
    let zoomFactor = global.camera.z;

    // Get texture size
    let textureSize = vec2<f32>(textureDimensions(t).xy);

    // Convert UV coordinates to pixel coordinates
    var pixelCoords = uv * textureSize;

    // Quantize the pixel coordinates
    var quantizedPixelCoords = floor(pixelCoords / zoomFactor) * zoomFactor;
    quantizedPixelCoords += zoomFactor * abs(fract(global.camera.xy)); // not sure if this does much tbh

    // Convert quantized pixel coordinates back to UV coordinates
    let quantizedUV = quantizedPixelCoords / textureSize;

    // Sample the texture at the quantized UV coordinates
    let quantizedColor = textureSample(t, s, quantizedUV);

    return quantizedColor;
}
