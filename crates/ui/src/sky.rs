use std::f32::consts::{FRAC_1_SQRT_2, PI};

use benthic_protocol::messages::ui::skybox_update::SkyboxUpdate;
use bevy::core_pipeline::Skybox;
use bevy::core_pipeline::prepass::DeferredPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::light_consts::lux;
use bevy::light::{
    Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, SunDisk, VolumetricFog,
};
use bevy::math::cubic_splines::LinearSpline;
use bevy::pbr::{AtmosphereSettings, DefaultOpaqueRendererMethod, ScreenSpaceReflections};
use bevy::prelude::*;
use bevy_post_process::auto_exposure::{
    AutoExposure, AutoExposureCompensationCurve, AutoExposurePlugin,
};
use bevy_post_process::bloom::Bloom;

use crate::render::MainCamera;

#[derive(Message)]
pub struct SkyboxUpdateEvent {
    pub value: SkyboxUpdate,
}

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct MoonLight;

#[derive(Resource)]
pub struct SunState {
    pub current_phase: f32,
    pub target_phase: f32,
}

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SkyboxUpdateEvent>()
            .insert_resource(DefaultOpaqueRendererMethod::deferred())
            .add_plugins(AutoExposurePlugin)
            .insert_resource(SunState {
                current_phase: 0.0,
                target_phase: 0.0,
            })
            .add_systems(Startup, setup_skybox)
            .add_systems(Update, (handle_skybox_update, update_sun))
            .insert_resource(SunState {
                current_phase: 0.0,
                target_phase: 0.0,
            });
    }
}

fn setup_skybox(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut compensation_curves: ResMut<Assets<AutoExposureCompensationCurve>>,
    camera: Single<Entity, With<MainCamera>>,
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
            illuminance: 0.065,
            color: Color::srgb(0.65, 0.7, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, -1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        MoonLight,
    ));

    commands.entity(*camera).insert((
        Atmosphere::earth(medium),
        AtmosphereSettings::default(),
        Skybox {
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
        AutoExposure {
            compensation_curve: under_expose_curve,
            speed_darken: 8.0,
            ..default()
        },
        DeferredPrepass,
        Msaa::Off,
        ScreenSpaceReflections::default(),
    ));
}

fn handle_skybox_update(mut events: MessageReader<SkyboxUpdateEvent>, mut sun: ResMut<SunState>) {
    for update in events.read() {
        sun.target_phase = update.value.sun_phase;
    }
}

fn update_sun(
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

        let adjusted_dir = (dir + Vec3::new(0.0, -FRAC_1_SQRT_2, FRAC_1_SQRT_2)) * 0.5;

        dir = (adjusted_dir * sun_dot + dir * (1.0 - sun_dot)).normalize();
    }

    for mut transform in &mut sun_query {
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, dir);
    }

    let moon_dir = -dir;

    for mut transform in &mut moon_query {
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, moon_dir);
    }
}
