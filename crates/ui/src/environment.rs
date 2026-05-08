use benthic_protocol::messages::ui::land_update::{LandData, LandUpdate};
use benthic_protocol::messages::ui::mesh_update::MeshType;
use benthic_protocol::messages::ui::skybox_update::SkyboxUpdate;
use benthic_protocol::messages::ui::water_update::WaterUpdate;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::BLACK;
use bevy::core_pipeline::prepass::DeferredPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::Skybox;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::light::light_consts::lux;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, FogVolume, SunDisk, VolumetricFog,
};
use bevy::math::cubic_splines::LinearSpline;
use bevy::pbr::{
    Atmosphere, AtmosphereSettings, ExtendedMaterial, MaterialExtension, ScatteringMedium,
    ScreenSpaceReflections,
};
use bevy::post_process::auto_exposure::{AutoExposure, AutoExposureCompensationCurve};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy_panorbit_camera::PanOrbitCamera;
use std::f32::consts::PI;
use std::fs;

use crate::render::{MainCamera, MeshQueue, Renderable, RenderableHandle};

#[derive(Message)]
pub struct WaterUpdateEvent {
    pub value: WaterUpdate,
}

#[derive(Message)]
pub struct SkyboxUpdateEvent {
    pub value: SkyboxUpdate,
}

#[derive(Message)]
pub struct LandUpdateEvent {
    pub value: LandUpdate,
}

#[derive(Component)]
pub struct WaterPlane;

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct MoonLight;

#[derive(Resource)]
pub struct SunState {
    pub current_phase: f32,
    pub target_phase: f32,
}

#[derive(Resource)]
pub struct AtmosphereHandle(pub Handle<ScatteringMedium>);

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut compensation_curves: ResMut<Assets<AutoExposureCompensationCurve>>,
    asset_server: Res<AssetServer>,
) {
    let under_expose_curve = compensation_curves.add(
        AutoExposureCompensationCurve::from_curve(LinearSpline::new([
            vec2(-4.0, 1.8 * 1.67),
            vec2(-2.0, 0.0 * 1.33),
            vec2(0.0, -2.0),
            vec2(2.0, -2.0 * 0.67),
            vec2(4.0, -2.0 * 0.33),
        ]))
        .expect("Failed to create compensation curve"),
    );

    let medium = scattering_mediums.add(ScatteringMedium::default());
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 15.0,
        ..default()
    }
    .build();
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        SunDisk::default(),
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight,
        cascade_shadow_config,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 0.065,
            color: Color::srgb(0.65, 0.7, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, -1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        MoonLight,
    ));
    commands.spawn((
        FogVolume::default(),
        Transform::from_scale(Vec3::new(10.0, 1.0, 10.0)).with_translation(Vec3::Y * 0.5),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.1, 0.1, 0.15),
        brightness: 1.0,
        affects_lightmapped_meshes: true,
    });

    let skybox_handle = asset_server.load("cubemaps/earth_starmap.ktx2");
    commands.spawn((
        PanOrbitCamera {
            radius: Some(3.0),
            ..Default::default()
        },
        AutoExposure {
            compensation_curve: under_expose_curve,
            speed_darken: 8.0,
            ..Default::default()
        },
        MainCamera,
        DeferredPrepass,
        Atmosphere::earthlike(medium.clone()),
        AtmosphereSettings::default(),
        Skybox {
            image: skybox_handle.clone(),
            brightness: 0.3,
            ..default()
        },
        AtmosphereEnvironmentMapLight::default(),
        Tonemapping::AgX,
        Bloom::NATURAL,
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        Msaa::Off,
        Fxaa::default(),
        ScreenSpaceReflections::default(),
    ));
    spawn_water(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut water_materials,
    );
}

pub fn handle_land_update(
    mut ev_land_update: MessageReader<LandUpdateEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_queue: ResMut<MeshQueue>,
) {
    for patch in ev_land_update.read() {
        let land = &patch.value;
        match fs::read_to_string(&land.path) {
            Ok(json_str) => match serde_json::from_str::<LandData>(&json_str) {
                Ok(land_data) => {
                    let mut mesh = Mesh::new(
                        bevy::mesh::PrimitiveTopology::TriangleList,
                        RenderAssetUsages::RENDER_WORLD,
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, land_data.vertices);
                    mesh.insert_indices(bevy::mesh::Indices::U16(land_data.indices));
                    mesh.compute_smooth_normals();
                    let mesh_handle = meshes.add(mesh);
                    mesh_queue.pending.push(Renderable {
                        handle: RenderableHandle::Mesh(mesh_handle),
                        transform: Transform {
                            translation: land_data.position,
                            ..Default::default()
                        },
                        parent: None,
                        mesh_type: MeshType::Land,
                        id: None,
                    })
                }
                Err(err) => error!("Failed to deserialize JSON: {}", err),
            },
            Err(err) => error!("Failed to read file {}", err),
        }
    }
}
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

fn spawn_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    water_materials: &mut Assets<ExtendedMaterial<StandardMaterial, Water>>,
) {
    let mut plane_mesh = Plane3d::new(Vec3::Y, Vec2::new(2000.0, 2000.0))
        .mesh()
        .build();
    plane_mesh.generate_tangents().unwrap();
    let mesh_handle = meshes.add(plane_mesh);
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: BLACK.into(),
                perceptual_roughness: 0.0,
                ..default()
            },
            extension: Water {
                normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                    "textures/water_normals.png",
                    |settings| {
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    },
                ),
                // These water settings are just random values to create some
                // variety.
                settings: WaterSettings {
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),
                        vec4(0.153, 0.138, -0.149, -0.195),
                    ],
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 500.0,
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044) * 0.2,
                },
            },
        })),
        Transform::from_scale(Vec3::splat(100.0)),
        WaterPlane,
    ));
}

pub fn handle_water_update(
    mut ev_water_update: MessageReader<WaterUpdateEvent>,
    mut water_query: Query<&mut Transform, With<WaterPlane>>,
) {
    for update in ev_water_update.read() {
        let height = update.value.height;

        for mut transform in &mut water_query {
            transform.translation.y = height;
            return;
        }
    }
}

pub fn handle_skybox_update(
    mut ev_skybox_update: MessageReader<SkyboxUpdateEvent>,
    mut sun: ResMut<SunState>,
) {
    for update in ev_skybox_update.read() {
        sun.target_phase = update.value.sun_phase as f32;
    }
}

pub fn update_sun(
    time: Res<Time>,
    mut sun: ResMut<SunState>,
    mut sun_query: Query<&mut Transform, With<SunLight>>,
    mut moon_query: Query<&mut Transform, (With<MoonLight>, Without<SunLight>)>,
) {
    let dt = time.delta_secs();
    let speed = 1.0;

    sun.current_phase = sun
        .current_phase
        .lerp(sun.target_phase, 1.0 - (-speed * dt).exp());

    let adjusted_phase = ((sun.current_phase / (2.0 * PI) + 0.25) % 1.0) * 24.0;
    let azimuth = (adjusted_phase / 24.0) * 2.0 * PI - PI / 2.0;
    let elevation = ((adjusted_phase - 6.0) / 12.0) * PI;

    let mut dir = Vec3::new(
        azimuth.cos() * elevation.cos(),
        azimuth.sin() * elevation.cos(),
        elevation.sin(),
    )
    .normalize();

    if dir.z > 0.0 {
        let sun_dot = dir.z * dir.z;
        let adjusted_dir = (dir + Vec3::new(0.0, -0.70711, 0.70711)) * 0.5;
        dir = (adjusted_dir * sun_dot + dir * (1.0 - sun_dot)).normalize();
    }

    for mut t in &mut sun_query {
        t.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, dir);
    }

    let moon_dir = -dir;

    for mut t in &mut moon_query {
        t.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, moon_dir);
    }
}
