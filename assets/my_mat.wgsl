#import bevy_render::view::View;

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct MyExtendedMaterial {
    quantize_steps: u32,
}

@group(2) @binding(100)
var<uniform> my_extended_material: MyExtendedMaterial;

@group(2) @binding(100)
var<uniform> view: View;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    //let normal = normalize(in.normal);

    // we can optionally modify the input before lighting and alpha_discard is applied
    //pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);
    
    var color = out.color.rgb;

    let steps = f32(my_extended_material.quantize_steps);
    color = floor(color * steps) / steps;
    color = max(color, vec3<f32>(0.001));

    // === RIM LIGHTING ===
    let normal = normalize(in.world_normal);
    let view_dir = normalize(view.world_position.xyz - in.world_position.xyz);
    let rim = 1.0 - dot(normal, view_dir);
    let rim_power = 3.0;
    let rim_intensity = pow(clamp(rim, 0.0, 1.0), rim_power);
    let rim_color = vec3<f32>(1.0); // White rim light
    color += rim_color * rim_intensity * 0.3; // Adjust rim strength

    out.color = vec4<f32>(color, out.color.a);

    // we can optionally modify the lit color before post-processing is applied
    //out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);


    // we can optionally modify the final result here
    out.color = out.color * 1.5;

    return out;
}
