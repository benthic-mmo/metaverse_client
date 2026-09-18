use benthic_protocol::messages::ui::water_update::WaterUpdate;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(Message)]
pub struct WaterUpdateEvent {
    pub value: WaterUpdate,
}

#[derive(Component)]
pub struct WaterPlane;

#[derive(ShaderType, Debug, Clone)]
struct WaterSettings {
    octave_vectors: [Vec4; 2],
    octave_scales: Vec4,
    octave_strengths: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Water {
    #[texture(100)]
    #[sampler(101)]
    normals: Handle<Image>,

    #[uniform(102)]
    settings: WaterSettings,
}

impl MaterialExtension for Water {
    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/water_material.wgsl".into()
    }
}

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WaterUpdateEvent>()
            .add_systems(Startup, setup_water)
            .add_systems(Update, handle_water_update)
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
    }
}

fn setup_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    asset_server: Res<AssetServer>,
) {
    let mut plane_mesh = Plane3d::new(Vec3::Y, Vec2::new(2000.0, 2000.0))
        .mesh()
        .build();

    plane_mesh.generate_tangents().unwrap();

    let mesh_handle = meshes.add(plane_mesh);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(
            water_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::BLACK,
                    perceptual_roughness: 0.0,
                    ..default()
                },
                extension: Water {
                    normals: asset_server
                        .load_builder()
                        .with_settings(|settings: &mut ImageLoaderSettings| {
                            settings.is_srgb = false;
                            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                                address_mode_u: ImageAddressMode::Repeat,
                                address_mode_v: ImageAddressMode::Repeat,
                                mag_filter: ImageFilterMode::Linear,
                                min_filter: ImageFilterMode::Linear,
                                ..default()
                            });
                        })
                        .load("textures/water_normals.png"),
                    settings: WaterSettings {
                        octave_vectors: [
                            vec4(0.080, 0.059, 0.073, -0.062),
                            vec4(0.153, 0.138, -0.149, -0.195),
                        ],
                        octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 500.0,
                        octave_strengths: vec4(0.16, 0.18, 0.093, 0.044) * 0.2,
                    },
                },
            }),
        ),
        Transform::from_scale(Vec3::splat(100.0)),
        WaterPlane,
    ));
}

fn handle_water_update(
    mut ev_water_update: MessageReader<WaterUpdateEvent>,
    mut water_query: Query<&mut Transform, With<WaterPlane>>,
) {
    for update in ev_water_update.read() {
        if let Some(mut transform) = water_query.iter_mut().next() {
            transform.translation.y = update.value.height;
        }
    }
}
