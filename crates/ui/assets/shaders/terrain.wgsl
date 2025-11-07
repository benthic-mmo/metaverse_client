#import bevy_pbr::forward_io::VertexOutput
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material_color: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var material_color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var material_color_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Generate a "UV" from world position (XZ planar mapping)
    let uv = mesh.world_position.xz * 0.003; // adjust 0.05 to scale/tiling

    // Sample texture using generated UV
    let tex_color = textureSample(material_color_texture, material_color_sampler, uv);

    return material_color * tex_color * COLOR_MULTIPLIER;
}
