// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::pbr::NotShadowReceiver;
use bevy::prelude::*;
use bevy::{asset::AssetMetaCheck, pbr::CascadeShadowConfigBuilder};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::f32::consts::PI;

static BG_COLOR: Color = Color::srgb(0.2, 0.76, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .insert_resource(ClearColor(BG_COLOR))
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, (setup, spawn_gltf, spawn_water))
        .add_systems(Update, simulate_tides)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
        FogSettings {
            color: BG_COLOR,
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 20.0,
            },
            ..default()
        },
    ));

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 20.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn spawn_gltf(mut commands: Commands, ass: Res<AssetServer>) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("island.glb#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(2.0, 0.0, -5.0),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct Water;

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.5, 0.7),
        reflectance: 0.3,
        ..Default::default()
    });

    let mesh_size = 50.0;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(mesh_size, mesh_size)
                    .subdivisions(1),
            ),
            material: water_material,
            ..default()
        },
        NotShadowReceiver,
        Water,
    ));
}

fn simulate_tides(time: Res<Time>, mut query: Query<&mut Transform, With<Water>>) {
    let wave_speed = 0.2;
    let wave_height = 0.3;

    for mut transform in query.iter_mut() {
        transform.translation.y = (time.elapsed_seconds() as f32 * wave_speed).sin() * wave_height;
    }
}
