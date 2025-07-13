struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
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
    return textureSample(texture, samp, in.uv);
}
