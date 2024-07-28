mod deserialize;
mod exports;
pub mod markov;

use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_rapier3d::prelude::*;
use deserialize::{setup_markov, GlbCollections};
use markov::{add_blocks_to_island, add_island, post_generate, WorldRandom, WorldState};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<GlbCollections>::new(&[
            "exports/data.json",
        ]))
        .init_state::<WorldState>()
        .init_resource::<WorldRandom>()
        // .add_systems(Startup, load_scene)
        .add_systems(Startup, setup_markov)
        .add_systems(Startup, setup_ground)
        .add_systems(Update, add_island.run_if(in_state(WorldState::Generating)))
        .add_systems(Update, add_blocks_to_island);
        // .add_systems(OnExit(WorldState::Generating), post_generate)
        // .add_systems(Update, scene_colliders);
    }
}

#[derive(Resource)]
pub struct MainScene {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

fn load_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MainScene {
        handle: assets.load("island.glb"),
        is_loaded: false,
    });
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_size = 500.0;
    let ground_transform = Transform::from_xyz(0.0, -0.5, 0.0);
    commands.spawn((
        ground_transform.clone(), // Position the ground slightly below the origin
        GlobalTransform::IDENTITY,
        Collider::cuboid(ground_size, 0.1, ground_size), // Create a large plane collider
        RigidBody::Fixed, // Make the collider fixed so it doesn't move
    ));

    // spawn in a mesh for the ground
    let plane = Plane3d::new(Vec3::new(0., 1.0, 0.), Vec2::new(ground_size, ground_size));
    let material = StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.5),
        ..Default::default()
    };
    // let mesh = Mesh::from(plane);
    // let mesh_
    commands.spawn(PbrBundle {
        mesh: meshes.add(plane),
        material: materials.add(material),
        transform: ground_transform,
        ..Default::default()
    });
}

fn scene_colliders(
    mut commands: Commands,
    mut main_scene: ResMut<MainScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if main_scene.is_loaded {
        return;
    }

    let gltf = gltf_assets.get(&main_scene.handle);

    if let Some(gltf) = gltf {
        let scene = gltf.scenes.first().unwrap().clone();

        commands.spawn(SceneBundle { scene, ..default() });
        for node in &gltf.nodes {
            let node = gltf_node_assets.get(node).unwrap();
            let bundles = spawn_collider_from_gltf_node(
                node,
                &gltf_mesh_assets,
                &mesh_assets,
                Transform::from_xyz(0.0, 0.0, 0.0),
            );
            for bundle in bundles {
                commands.spawn(bundle);
            }
        }
        main_scene.is_loaded = true;
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    collider: Collider,
    // transform_bundle: TransformBundle,
}

pub fn spawn_collider_from_gltf_node(
    gltf_node: &GltfNode,
    gltf_mesh_assets: &Assets<GltfMesh>,
    mesh_assets: &Assets<Mesh>,
    transform: Transform,
) -> Vec<ColliderBundle> {
    let mut bundles: Vec<ColliderBundle> = Vec::new();

    // instead build a compund shape here https://rapier.rs/docs/user_guides/bevy_plugin/colliders#compound-shapes
    // or attach all as children?

    if let Some(gltf_mesh) = gltf_node.mesh.clone() {
        let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
        for mesh_primitive in gltf_mesh.primitives.iter() {
            let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();

            bundles.push(ColliderBundle {
                collider: Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
                // transform_bundle: TransformBundle::from_transform(gltf_node.transform * transform),
            });
        }
    }

    bundles
}
