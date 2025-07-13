struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    time: f32,
    aspect: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var samp: sampler;

@vertex
fn vs_main(model: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var pi = 3.141592653589793;

    var uv_float = vec2<f32>(in.uv.x + cos(u.time * 0.45 + 2.0 * pi * in.uv.y) * 0.05, in.uv.y + sin(u.time * 0.6 + 2.0 * pi * in.uv.x) * 0.06);

    return textureSample(texture, samp, uv_float);
}
