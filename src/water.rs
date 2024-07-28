use bevy::{pbr::NotShadowReceiver, prelude::*};
use bevy_fps_controller::controller::{LogicalPlayer, RenderPlayer};
use bevy_rapier3d::prelude::{Collider, Velocity};

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_water).add_systems(
            Update,
            (simulate_tides, float_on_water_system, update_underwater_fog),
        );
    }
}

#[derive(Component)]
struct Water;

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.2, 0.4),
        reflectance: 0.3,
        // alpha_mode: AlphaMode::Add,
        perceptual_roughness: 0.0,
        double_sided: true,
        ..Default::default()
    });

    let mesh_size = 500.0;
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
    let wave_height = 16.0;
    let offset = 18.0;

    for mut transform in query.iter_mut() {
        transform.translation.y =
            offset + (time.elapsed_seconds() * wave_speed).sin() * wave_height;
    }
}

fn float_on_water_system(
    water_query: Query<&Transform, With<Water>>,
    // mut player_query: Query<(&mut Transform, &mut Velocity), With<LogicalPlayer>>,
    mut player_query: Query<(&mut Collider, &mut Velocity), With<LogicalPlayer>>,
    render_player_query: Query<&Transform, With<RenderPlayer>>,
) {
    if let Ok(water_transform) = water_query.get_single() {
        let water_level = water_transform.translation.y;
        //     let dt = time.delta_seconds();

        if let Ok((mut collider, mut velocity)) = player_query.get_single_mut() {
            if let Ok(player_transform) = render_player_query.get_single() {
                //
                let player_is_underwater = player_transform.translation.y < water_level;

                if player_is_underwater {
                    // Apply a buoyancy force to float the character on the water surface
                    velocity.linvel.y += 9.8 * 0.05; // Adjust buoyancy force as needed
                }
            }

            //         let collider_height = transform.translation.y;

            //         let character_bottom = transform.translation.y - collider_height / 2.0;
            //         if character_bottom < water_level {
            //             // Adjust character to float on the water surface
            //             transform.translation.y += (water_level - character_bottom) * dt;

            //             // Apply a damping force to simulate water resistance
            //             velocity.linvel *= 0.9; // Adjust damping factor as needed
            //         }
        }
    }
}

fn update_underwater_fog(
    water_query: Query<&Transform, With<Water>>,
    mut fog_query: Query<(&mut FogSettings, &Transform), With<RenderPlayer>>,
) {
    if let Ok(water_transform) = water_query.get_single() {
        let water_level = water_transform.translation.y;

        for (mut fog_settings, player_transform) in fog_query.iter_mut() {
            // if underwater, enable fog, otherwise, disable fog
            let player_is_underwater = player_transform.translation.y < water_level;
            if player_is_underwater {
                fog_settings.falloff = FogFalloff::Exponential { density: // density increases as player goes deeper
                    (water_level - player_transform.translation.y) / 10.0,
                }
            } else {
                fog_settings.falloff = FogFalloff::Exponential { density: 0.0 }
            }
        }
    }
}
