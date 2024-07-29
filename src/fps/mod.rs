use bevy::{prelude::*, render::camera::Exposure, window::CursorGrabMode};
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;

use crate::BG_COLOR;

const SPAWN_POINT: Vec3 = Vec3::new(15.0, 12.625, 15.0);

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsControllerPlugin)
            .add_systems(Startup, setup)
            // .add_systems(Startup, load_bike_scene)
            // .add_systems(Startup, spawn_handle_bars)
            .add_systems(Update, (manage_cursor, respawn));
        // .add_systems(Update, update_handle_bars);
    }
}

#[derive(Resource)]
pub struct BikeModel {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

// fn load_bike_scene(mut commands: Commands, assets: Res<AssetServer>) {
//     commands.insert_resource(BikeModel {
//         handle: assets.load("bike.glb"),
//         is_loaded: false,
//     });
// }

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let mut window = window.single_mut();
    window.title = String::from("Strawberry Jam");

    // Note that we have two entities for the player
    // One is a "logical" player that handles the physics computation and collision
    // The other is a "render" player that is what is displayed to the user
    // This distinction is useful for later on if you want to add multiplayer,
    // where often time these two ideas are not exactly synced up
    let height = 2.0;
    let logical_entity = commands
        .spawn((
            Collider::cylinder(height / 2.0, 0.5),
            // A capsule can be used but is NOT recommended
            // If you use it, you have to make sure each segment point is
            // equidistant from the translation of the player transform
            // Collider::capsule_y(height / 2.0, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,

                ..default()
            },
            FpsController {
                air_acceleration: 80.0,
                key_left: KeyCode::KeyU,
                key_right: KeyCode::KeyU,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id();

    commands
        .spawn((
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: TAU / 5.0,
                    ..default()
                }),
                exposure: Exposure::INDOOR,
                ..default()
            },
            FogSettings {
                color: BG_COLOR,
                falloff: FogFalloff::Exponential { density: 0.005 },
                ..default()
            },
            RenderPlayer { logical_entity },
        ))
        .with_children(|children| {
            // spawn bike!
            let cyclinder = Cylinder::new(0.05, 1.0);
            let mesh = meshes.add(Mesh::from(cyclinder));
            let material = StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.5),
                ..Default::default()
            };

            // we want the cycliner to be 'handle bars' so we rotate it 90 degrees around the x-axis and then move it forward a bit
            let transform = Transform::from_rotation(Quat::from_rotation_z(TAU / 4.0));
            let transform = transform * Transform::from_translation(Vec3::new(0.00, 0.00, -0.45));

            let scene_handle: Handle<Scene> = assets.load("bike.glb#Scene0");

            // children.spawn(PbrBundle {
            //     mesh,
            //     material: materials.add(material),
            //     transform,
            //     ..default()
            // });

            children.spawn(SceneBundle {
                scene: scene_handle,
                transform: Transform::from_xyz(0.0, -1.6, -0.8),
                ..Default::default()
            });
        });
}

fn respawn(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.linvel = Vec3::ZERO;
        transform.translation = SPAWN_POINT;
    }
}

fn manage_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut FpsController>,
) {
    for mut window in &mut window_query {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
            for mut controller in &mut controller_query {
                controller.enable_input = true;
            }
        }
        if key.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            for mut controller in &mut controller_query {
                controller.enable_input = false;
            }
        }
    }
}

// #[derive(Component)]
// struct HandleBars;

// fn spawn_handle_bars(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let cyclinder = Cylinder::new(0.05, 1.0);
//     let mesh = meshes.add(Mesh::from(cyclinder));

//     let material = StandardMaterial {
//         base_color: Color::srgb(0.5, 0.5, 0.5),
//         ..Default::default()
//     };

//     // transform should be 90 degrees rotated around the x-axis
//     let transform = Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0));

//     commands
//         .spawn(PbrBundle {
//             mesh,
//             material: materials.add(material),
//             transform,
//             ..Default::default()
//         })
//         .insert(HandleBars);
// }

// place handlebars in front of the player
// fn update_handle_bars(
//     // query: Query<(&Transform, &RenderPlayer)>,
//     render_player_query: Query<&GlobalTransform, With<LogicalPlayer>>,
//     mut handle_bars_query: Query<&mut Transform, With<HandleBars>>,
// ) {
//     for render_player_transform in &render_player_query {
//         if let Ok(mut handle_bars_transform) = handle_bars_query.get_single_mut() {
//             let player_transform = render_player_transform.compute_transform();

//             let new_handle_bars_transform = render_player_transform.compute_transform()
//                 * Transform::from_translation(Vec3::new(0.0, 0.0, 1.));
//             *handle_bars_transform = new_handle_bars_transform;
//         }
//     }
// }

// mut player_query: Query<(&mut Collider, &mut Velocity), With<LogicalPlayer>>,
//     render_player_query: Query<&Transform, With<RenderPlayer>>,

// .with_children(|children| {
//     // spawn bike!
//     let cyclinder = Cylinder::new(0.1, 0.5);
//     let mesh = meshes.add(Mesh::from(cyclinder));

//     let material = StandardMaterial {
//         base_color: Color::srgb(0.5, 0.5, 0.5),
//         ..Default::default()
//     };

//     // transform should be 90 degrees rotated around the x-axis
//     let transform = Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0));

//     // then move it forward a bit so we can see it
//     let transform = transform * Transform::from_translation(Vec3::new(0.0, -0.0, 0.)); //.25

//     children.spawn(PbrBundle {
//         mesh,
//         material: materials.add(material),
//         transform,
//         ..default()
//     });
// });
