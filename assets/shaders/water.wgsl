#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::prepass_utils

struct WaterMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: WaterMaterial;
// @group(1) @binding(1)
// var base_color_texture: texture_2d<f32>;
// @group(1) @binding(2)
// var base_color_sampler: sampler;


fn gerstner(pos: vec3<f32>, dir: vec2<f32>, steepness: f32, wave_len: f32, time: f32, wave_height: f32) -> vec3<f32> {
    let k = 2.0 * 3.141592 / wave_len;
    let c = sqrt(9.8 / k);
    let d = normalize(dir);
    let f = k * (dot(d, pos.xz) - c * time);
    let a = steepness / k;
    return vec3<f32>(d.x * ((a * cos(f)) * 0.5 + 0.5), a * (sin(f) * 0.5 + 0.5), d.y * (a * cos(f)) * 0.5 + 0.5) * wave_height;
}

fn mod289(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2<f32>) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

fn rand2(n: vec2<f32>) -> f32 {
  return fract(sin(dot(n, vec2<f32>(12.9898, 4.1414))) * 43758.5453);
}

fn noise2(n: vec2<f32>) -> f32 {
//   let d = vec2<f32>(0., 1.);
//   let b = floor(n);
//   let f = smoothstep(vec2<f32>(0.), vec2<f32>(1.), fract(n));
//   return mix(mix(rand2(b), rand2(b + d.yx), f.x), mix(rand2(b + d.xy), rand2(b + d.yy), f.x), f.y);
    return simplexNoise2(n);
}

//  MIT License. © Inigo Quilez, Munrocket
//  noise2() is any noise here: Value, Perlin, Simplex, Worley
//
fn fbm(p: vec2<f32>) -> f32 {
    var m2: mat2x2<f32> = mat2x2<f32>(vec2<f32>(0.8, 0.6), vec2<f32>(-0.6, 0.8));
    var f: f32 = 0.;
    f = f + 0.5000 * noise2(p);
    var p = m2 * p * 2.02;
    f = f + 0.2500 * noise2(p);
    p = m2 * p * 2.03;
    f = f + 0.1250 * noise2(p);
    p = m2 * p * 2.01;
    f = f + 0.0625 * noise2(p);
    return f / 0.9375;
}


@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(sample_index) sample_index: u32,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
) -> @location(0) vec4<f32> {
    let shallow = vec4(0.2, 0.9, 0.8, 0.6);
    let deep = vec4(0.0, 0.2, 0.5, 0.8);

    let depth = prepass_depth(frag_coord, sample_index);
    let dist = view.world_position - world_position.xyz;
    let output = length(vec3(depth)-dist);
    let V = normalize(dist);
    let fresnel = 1.0-dot(world_normal, V);

    // whack attempt, looks alright
    let mask1 = 1.0 - smoothstep(0.0, min(length(dist * 12.0), 1.0), output * depth * 12.0 * fresnel );

    // more accurate?
    let mask2 = saturate(1.0 - (frag_coord.z - depth) * 13.0 - 0.2);

    let adjust = depth / frag_coord.z;
    let mask3 = 1.0 - (pow(adjust,1.5) * 1.2) * (pow(fresnel, 2.0) * 2.0);

    let ab_color = vec3(1.0) - deep.xyz;
    let ab_val = 1.0 - pow(adjust, 2.0);
    let sub_color = ab_color * ab_val;

    let near = 0.1;
    let far = 1000.0;
    let floordistance = 2.0 * near * far / (far + near - (2.0 * depth - 1.0) * (far - near));
    let surfacedistance = 2.0 * near * far / (far + near - (2.0 * frag_coord.z - 1.0) * (far - near));

    let test = smoothstep(0.0, 1.0, (frag_coord.z - depth) * pow(length(dist), 2.0) * 10.0);

    //return vec4(mix(deep, shallow,  test));
    if (frag_coord.z > 1.0){
        discard;
    }
    let test2 = saturate(floordistance - frag_coord.z);
    return vec4(vec3(sub_color), 1.0);

}
