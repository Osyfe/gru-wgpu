struct VSInput
{
    @location(0) pos: vec2<f32>,
    @location(1) col: vec4<f32>,
    @location(2) coords: vec2<f32>,
    @location(3) layer: i32,
};

struct VSOutput
{
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec4<f32>,
    @location(1) coords: vec2<f32>,
    @location(2) @interpolate(flat) layer: i32,
}

@vertex
fn vs_main(in: VSInput) -> VSOutput
{
    return VSOutput(vec4<f32>(in.pos, 0.0, 1.0), in.col, in.coords, in.layer);
}

@group(0) @binding(0)
var t_glyphs: texture_2d_array<f32>;
@group(0) @binding(1)
var s_glyphs: sampler;

fn contour(d: f32, w: f32) -> f32
{
    return smoothstep(0.5 - w, 0.5 + w, d);
}

fn samp(uv: vec2<f32>, layer: i32, w: f32) -> f32
{
    return contour(textureSample(t_glyphs, s_glyphs, uv, layer).r, w);
}

fn srgb2rgb(srgb: f32) -> f32
{
    if srgb <= 0.04045 { return srgb / 12.92; }
    else { return pow((srgb + 0.055) / 1.055, 2.4); }
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32>
{
    var layer = max(in.layer, 0);
    var uv = in.coords;
    var dist: f32 = textureSample(t_glyphs, s_glyphs, uv, layer).r;
    var width = fwidth(dist);
    var alpha = contour(dist, width);
    var dscale = 0.354; //half of 1/sqrt2
    var duv = dscale * (dpdx(uv) + dpdy(uv));
    var box = vec4<f32>(uv - duv, uv + duv);
    var asum =
        samp(box.xy, layer, width)
        + samp(box.zw, layer, width)
        + samp(box.xw, layer, width)
        + samp(box.zy, layer, width);
    alpha = (alpha + 0.5 * asum) / 3.0;
    alpha = 1.0 - srgb2rgb(1.0 - alpha);

    if in.layer == -1 { return in.col; }
    else { return vec4<f32>(in.col.rgb, alpha * in.col.a); }
}
