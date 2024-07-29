// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    dead_code,
    // unused_imports
)]

mod fps;
mod water;
mod world;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::{asset::AssetMetaCheck, pbr::CascadeShadowConfigBuilder};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::*;
use fps::FpsPlugin;
use water::WaterPlugin;
use world::markov::BlockInstanceCollider;
use world::WorldPlugin;

const BG_COLOR: Color = Color::WHITE;
const BG_VALUE: f32 = 0.75;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(FpsPlugin)
        .add_plugins(WorldPlugin)
        // .add_plugins(WaterPlugin)
        // .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::srgb(BG_VALUE, BG_VALUE, BG_VALUE)))
        .add_systems(Startup, setup_sun)
        // .add_systems(Startup, setup_pan_camera)
        // .add_systems(
        //     Update,
        //     on_enter_turn_fixed_rigid_bodies_into_static_rigid_bodies,
        // )
        // .add_systems(Update, rotate_sun)
        .run();
}

fn setup_pan_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera {
            button_pan: MouseButton::Left,
            modifier_pan: Some(KeyCode::ShiftLeft),
            ..default()
        },
    ));
}

// derive sun component
#[derive(Component)]
pub struct Sun;

fn setup_sun(mut commands: Commands) {
    // directional 'sun' light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: light_consts::lux::OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 200.0,
                ..default()
            }
            .into(),
            ..default()
        },
        Sun,
    ));
}

fn rotate_sun(
    time: Res<Time>,
    mut sun_transform: Query<&mut Transform, With<Sun>>,
    mut sun_light: Query<&mut DirectionalLight, With<Sun>>,
) {
    for mut transform in sun_transform.iter_mut() {
        transform.rotation = Quat::from_rotation_y(time.elapsed_seconds() as f32);
    }

    for mut light in sun_light.iter_mut() {
        // light.illuminance =

        //     // sin(time.seconds_since_startup() as f32).abs() * light_consts::lux::FULL_DAYLIGHT;

        // use bevy math sin wave
        let sin_wave = (time.elapsed_seconds_wrapped() as f32).sin();
        light.illuminance = (sin_wave * 0.5 + 0.5) * light_consts::lux::FULL_DAYLIGHT;
    }

    // the sun strength should be based on the time of day
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

fn on_enter_turn_fixed_rigid_bodies_into_static_rigid_bodies(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    rigid_body_query: Query<(Entity, &RigidBody), With<BlockInstanceCollider>>,
) {
    let rigid_body: Option<RigidBody>;

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        rigid_body = Some(RigidBody::Dynamic);
    } else if keyboard_input.just_pressed(KeyCode::KeyO) {
        rigid_body = Some(RigidBody::Fixed);
    } else {
        return;
    }

    for (entity, body) in rigid_body_query.iter() {
        if *body != rigid_body.unwrap() {
            // println!("rigid body is fixed");

            commands.entity(entity).remove::<RigidBody>();
            commands.entity(entity).insert(rigid_body.unwrap());
        }
    }
}
