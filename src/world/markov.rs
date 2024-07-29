use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_rapier3d::{
    plugin::RapierContext,
    prelude::{Collider, QueryFilter, RigidBody},
};
use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::world::spawn_collider_from_gltf_node;

use super::deserialize::{GlbCollections, MarkovCollection};

pub fn post_generate(rapier_context: Res<RapierContext>) {
    println!("length of colliders: {:?}", rapier_context.colliders.len());
}

pub fn add_island(
    mut commands: Commands,
    mut world_state: ResMut<NextState<WorldState>>,
    mut world_random: ResMut<WorldRandom>,
    collections: Res<Assets<GlbCollections>>,
    markov_collection: Res<MarkovCollection>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if gltf_assets.len() == 0 {
        return;
    }
    if let Some(_) = collections.get(&markov_collection.collections_handle) {
        println!("Generating world");

        let count = 40;
        for i in 0..count {
            println!("Generating island number: {:?}", i);

            // start by randomly picking a glb and spawning at 0,0
            let random_index = world_random
                .0
                .gen_range(0..markov_collection.glb_assets.len());
            let random_instance = markov_collection
                .glb_assets
                .keys()
                .nth(random_index)
                .unwrap();

            let size = 200.0;
            let xform = Transform::from_xyz(
                world_random.0.gen_range(-size..size),
                0.0,
                world_random.0.gen_range(-size..size),
            );

            let gltf_handle = markov_collection.glb_assets.get(random_instance).unwrap();
            println!(
                "random instance: {:?}, handle {:?}",
                random_instance, gltf_handle
            );
            let gltf = gltf_assets.get(gltf_handle).unwrap();

            let scene = gltf.scenes.first().unwrap().clone();
            commands
                .spawn((
                    SceneBundle {
                        scene,
                        transform: xform.clone(),
                        ..Default::default()
                    },
                    BlockInstance {
                        name: random_instance.clone(),
                    },
                    RigidBody::Fixed,
                    BlockInstanceCollider,
                ))
                .with_children(|children| {
                    for node in &gltf.nodes {
                        let node = gltf_node_assets.get(node).unwrap();
                        for bundle in
                            spawn_collider_from_gltf_node(node, &gltf_mesh_assets, &mesh_assets)
                        {
                            children.spawn(bundle);
                        }
                    }
                });

            let island_size = world_random.0.gen_range(20..80) as usize;
            commands.spawn((Island::new(island_size), xform));

            world_state.set(WorldState::Generated);
        }
    } else {
        println!("No collections found");
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum WorldState {
    #[default]
    Generating,
    Generated,
}

#[derive(Component)]
pub struct BlockInstance {
    pub name: String,
}

#[derive(Resource)]
pub struct WorldRandom(pub StdRng);

impl Default for WorldRandom {
    fn default() -> Self {
        WorldRandom(StdRng::from_entropy())
    }
}

#[derive(Component)]
pub struct Island {
    pub children: Vec<Entity>,
    pub size: usize,
}

impl Island {
    fn new(size: usize) -> Self {
        Island {
            children: Vec::new(),
            size,
        }
    }
}

// pub fn press_enter_to_add_island(keyboard_input: Res<ButtonInput<KeyCode>>) {
//     if !keyboard_input.just_pressed(KeyCode::Enter) {
//         return;
//     }
// }

pub fn add_blocks_to_island(
    mut commands: Commands,
    block_instance_query: Query<(&BlockInstance, &Transform)>,
    mut random: ResMut<WorldRandom>,
    markov_collection: Res<MarkovCollection>,
    collections: Res<Assets<GlbCollections>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
    rapier_context: Res<RapierContext>,
    mut island_query: Query<&mut Island>,
) {
    for mut island in island_query.iter_mut() {
        if island.children.len() > island.size {
            return;
        }

        let collection = collections
            .get(&markov_collection.collections_handle)
            .unwrap();
        let mut transforms: Vec<(&BlockInstance, &Transform)> =
            block_instance_query.iter().collect();

        if let Some((block_instance, transform)) = transforms.choose_mut(&mut random.0) {
            // given the transform and the block_instance, what is likely to be the next block_instance?

            let next_transforms = &collection.0.get(&block_instance.name).unwrap().transforms;
            let next_transform = next_transforms.choose(&mut random.0).unwrap();

            let xform = Transform::from_matrix(Mat4::from_cols_array_2d(&next_transform.1 .0));
            let xform = transform.clone() * xform;

            let gltf_handle = markov_collection.glb_assets.get(&next_transform.0).unwrap();
            let gltf = gltf_assets.get(gltf_handle).unwrap();
            let scene = gltf.scenes.first().unwrap().clone();

            let can_place = {
                let aabb = collection.0.get(&next_transform.0).unwrap().aabb;

                let mut shape_pos = xform.translation.clone();

                shape_pos += Vec3::new(aabb.center[0], aabb.center[1], aabb.center[2]);

                let shape_rot = xform.rotation.clone();

                let tolerance = 0.05;
                let hx = aabb.half_extents[0] - tolerance;
                let hy = aabb.half_extents[1] - tolerance; // y and z are swapped
                let hz = aabb.half_extents[2] - tolerance;

                let shape = Collider::cuboid(hx, hy, hz);
                let filter = QueryFilter::default();

                let mut can_place = true;
                rapier_context.intersections_with_shape(
                    shape_pos,
                    shape_rot,
                    &shape,
                    filter,
                    |entity| {
                        // println!("The entity {:?} intersects our shape.", entity);
                        can_place = false;
                        false
                    },
                );

                can_place
            };

            if can_place {
                let mut entity = commands.spawn((
                    SceneBundle {
                        scene,
                        transform: xform.clone(),
                        ..Default::default()
                    },
                    BlockInstance {
                        name: next_transform.0.clone(),
                    },
                    RigidBody::Fixed,
                    BlockInstanceCollider,
                ));

                entity.with_children(|children| {
                    for node in &gltf.nodes {
                        let node = gltf_node_assets.get(node).unwrap();
                        for bundle in
                            spawn_collider_from_gltf_node(node, &gltf_mesh_assets, &mesh_assets)
                        {
                            children.spawn(bundle);
                        }
                    }
                });

                island.children.push(entity.id());
            }
        }
    }
}

#[derive(Component)]
pub struct BlockInstanceCollider;
