use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    reflect::TypePath,
    utils::HashMap,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::exports;
use super::{exports::EXPORTED_FILES, markov};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformMatrix(pub [[f32; 4]; 4]);

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct AABB {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionData {
    file: String,
    pub aabb: AABB,
    pub transforms: Vec<(String, TransformMatrix)>,
}

#[derive(Serialize, Deserialize, Debug, Asset, TypePath)]
pub struct GlbCollections(pub HashMap<String, CollectionData>);

#[derive(Resource)]
pub struct MarkovCollection {
    pub collections_handle: Handle<GlbCollections>,
    pub glb_assets: HashMap<String, Handle<Gltf>>,
}

pub fn setup_markov(mut commands: Commands, asset_server: Res<AssetServer>) {
    let collections: Handle<GlbCollections> = asset_server.load("exports/data.json");
    let mut glb_assets: HashMap<String, Handle<Gltf>> = HashMap::new();

    EXPORTED_FILES.iter().for_each(|file| {
        let path_name = format!("exports/{}", file);
        let path_name = format!("{}.glb", path_name);
        println!("Loading {}", path_name);
        let glb_handle: Handle<Gltf> = asset_server.load(path_name);
        glb_assets.insert(file.to_string(), glb_handle);
    });

    commands.insert_resource(MarkovCollection {
        collections_handle: collections,
        glb_assets,
    });
}
