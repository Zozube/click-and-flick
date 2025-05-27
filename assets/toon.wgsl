struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> LightDirection: vec3<f32>;

@group(1) @binding(1)
var<uniform> BaseColor: vec4<f32>;

@vertex
fn vertex_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(position, 1.0);
    out.world_pos = position;
    out.normal = normalize(normal);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(LightDirection);
    let ndotl = max(dot(in.normal, light_dir), 0.0);

    // Cel shading: 3-band light steps
    let shade = select(
        select(0.2, 0.5, ndotl > 0.25),
        1.0,
        ndotl > 0.75
    );

    // Gold tone with cartoon style
    let base_color = BaseColor.rgb;
    return vec4(base_color * shade, 1.0);
}
