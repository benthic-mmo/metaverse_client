use bevy::{
    asset::AssetServer,
    camera::{Camera3d, visibility::Visibility},
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        hierarchy::ChildOf,
        name::Name,
        query::{Changed, With},
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    gltf::GltfAssetLabel,
    light::DirectionalLight,
    math::{Dir3, Isometry3d},
    mesh::skinning::SkinnedMesh,
    text::{TextColor, TextFont},
    transform::components::{GlobalTransform, Transform},
    ui::{
        AlignItems, BackgroundColor, FlexDirection, Interaction, JustifyContent, Node,
        PositionType, Val,
        widget::{Button, Text},
    },
};
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_world_serialization::WorldAssetRoot;
use glam::Vec3;

use crate::common::CreationArtifacts;

#[derive(Resource, Default)]
pub struct DebugVisibility {
    pub show_mesh: bool,
    pub show_skeleton: bool,
    pub show_joint_axes: bool,
}

type ToggleButtonsQueryData<'a> = (
    &'a Interaction,
    &'a mut BackgroundColor,
    Option<&'a ToggleMeshButton>,
    Option<&'a ToggleSkeletonButton>,
    Option<&'a ToggleJointAxesButton>,
);

type ToggleButtonsQueryFilter = (Changed<Interaction>, With<Button>);

#[derive(Component)]
pub struct ToggleMeshButton;

#[derive(Component)]
pub struct ToggleSkeletonButton;

#[derive(Component)]
pub struct ToggleJointAxesButton;

pub fn handle_ui_buttons(
    mut interaction_query: Query<ToggleButtonsQueryData, ToggleButtonsQueryFilter>,
    mut debug_visibility: ResMut<DebugVisibility>,
) {
    for (interaction, mut bg_color, is_mesh, is_skeleton, is_joint_axes) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            bg_color.0 = Color::srgb(0.4, 0.4, 0.4);

            if is_mesh.is_some() {
                debug_visibility.show_mesh = !debug_visibility.show_mesh;
            }
            if is_skeleton.is_some() {
                debug_visibility.show_skeleton = !debug_visibility.show_skeleton;
            }
            if is_joint_axes.is_some() {
                debug_visibility.show_joint_axes = !debug_visibility.show_joint_axes;
            }
        } else if *interaction == Interaction::Hovered {
            bg_color.0 = Color::srgb(0.3, 0.3, 0.3);
        } else {
            bg_color.0 = Color::srgb(0.2, 0.2, 0.2);
        }
    }
}

pub fn update_mesh_visibility(
    debug_visibility: Res<DebugVisibility>,
    mut mesh_query: Query<&mut Visibility, With<SkinnedMesh>>,
) {
    if debug_visibility.is_changed() {
        let target_visibility = if debug_visibility.show_mesh {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        for mut visibility in &mut mesh_query {
            *visibility = target_visibility;
        }
    }
}
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    artifacts: Res<CreationArtifacts>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.0, 5.0),
        PanOrbitCamera {
            focus: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        },
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 5.0, 3.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        WorldAssetRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(artifacts.gltf_object.clone())),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Combined"),
        Visibility::Visible,
    ));

    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..Default::default()
        },))
        .with_children(|parent| {
            // Toggle Mesh Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ToggleMeshButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Toggle Mesh"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ToggleSkeletonButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Toggle Skeleton"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ToggleJointAxesButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Toggle Joint Axes"),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn draw_skeleton_debug(
    mut gizmos: Gizmos,
    debug_visibility: Res<DebugVisibility>,
    skinned_mesh_query: Query<&SkinnedMesh>,
    parent_query: Query<&ChildOf>,
    global_transform_query: Query<&GlobalTransform>,
) {
    if !debug_visibility.show_skeleton {
        return;
    }

    for skinned_mesh in &skinned_mesh_query {
        for &joint_entity in &skinned_mesh.joints {
            if let Ok(joint_transform) = global_transform_query.get(joint_entity) {
                let joint_pos = joint_transform.translation();

                // Joint position
                gizmos.sphere(
                    Isometry3d::from_translation(joint_pos),
                    0.02,
                    Color::srgb(0.0, 1.0, 0.0),
                );

                // Parent -> joint bone
                if let Ok(parent_component) = parent_query.get(joint_entity) {
                    let parent_entity = parent_component.parent();

                    if skinned_mesh.joints.contains(&parent_entity)
                        && let Ok(parent_transform) = global_transform_query.get(parent_entity)
                    {
                        let parent_pos = parent_transform.translation();

                        gizmos.line(parent_pos, joint_pos, Color::srgb(1.0, 1.0, 0.0));
                    }
                }

                // Joint local orientation axes
                if debug_visibility.show_joint_axes {
                    let rotation = joint_transform.rotation();
                    let axis_length = 0.1;

                    let x = rotation * Vec3::X * axis_length;
                    let y = rotation * Vec3::Y * axis_length;
                    let z = rotation * Vec3::Z * axis_length;

                    // X = red
                    gizmos.line(joint_pos, joint_pos + x, Color::srgb(1.0, 0.0, 0.0));

                    // Y = green
                    gizmos.line(joint_pos, joint_pos + y, Color::srgb(0.0, 1.0, 0.0));

                    // Z = blue
                    gizmos.line(joint_pos, joint_pos + z, Color::srgb(0.0, 0.0, 1.0));
                }
            }
        }
    }
}
