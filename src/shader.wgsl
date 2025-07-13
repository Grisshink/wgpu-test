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

struct Colors {
    bg_color: vec3<f32>,
    _pad: u32,
    fg_color: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(0) @binding(1)
var<uniform> c: Colors;

@vertex
fn vs_main(model: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.pos, 0.0, 1.0);
    return out;
}

fn rand_3d(co: vec3<f32>) -> f32 {
     return fract(sin(dot(co.xyz, vec3<f32>(12.9898,78.233,144.7272))) * 43758.5453);
}

fn perlin_layer(co: vec3<f32>) -> f32 {
    var quant_floor = floor(co);
    var quant_ceil = ceil(co);
    var quant_frac = fract(co);

    var bot_left_back   = rand_3d(vec3<f32>(quant_floor.x, quant_floor.y, quant_floor.z));
    var bot_right_back  = rand_3d(vec3<f32>(quant_ceil.x, quant_floor.y, quant_floor.z));
    var top_left_back   = rand_3d(vec3<f32>(quant_floor.x, quant_ceil.y, quant_floor.z));
    var top_right_back  = rand_3d(vec3<f32>(quant_ceil.x, quant_ceil.y, quant_floor.z));

    var bot_left_front  = rand_3d(vec3<f32>(quant_floor.x, quant_floor.y, quant_ceil.z));
    var bot_right_front = rand_3d(vec3<f32>(quant_ceil.x, quant_floor.y, quant_ceil.z));
    var top_left_front  = rand_3d(vec3<f32>(quant_floor.x, quant_ceil.y, quant_ceil.z));
    var top_right_front = rand_3d(vec3<f32>(quant_ceil.x, quant_ceil.y, quant_ceil.z));

    var bot_back = mix(bot_left_back, bot_right_back, quant_frac.x);
    var top_back = mix(top_left_back, top_right_back, quant_frac.x);

    var bot_front = mix(bot_left_front, bot_right_front, quant_frac.x);
    var top_front = mix(top_left_front, top_right_front, quant_frac.x);

    var back = mix(bot_back, top_back, quant_frac.y);
    var front = mix(bot_front, top_front, quant_frac.y);

    return mix(back, front, quant_frac.z);
}

fn perlin(co: vec3<f32>) -> f32 {
     var out_val = 0.0;
     for (var i = 0; i < 8; i++) {
         out_val += perlin_layer(co * pow(2.0, f32(i)));
     }
     return out_val / 8.0;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var uv_float = vec2<f32>(in.uv.x * u.aspect, in.uv.y) * 5.0;
    var value = perlin(vec3<f32>(uv_float, u.time * 0.1));

    var value_s: f32;
    if value > 0.58 {
        value_s = 1.0;
    } else if value > 0.53 {
        value_s = 0.25;
    } else {
        value_s = 0.0;
    }

    var color = mix(c.bg_color, c.fg_color, value_s);
    return vec4<f32>(color, 1.0);
}
