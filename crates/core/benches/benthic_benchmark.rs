use benthic_protocol::render_data::RenderObject;
use criterion::{Criterion, criterion_group, criterion_main};
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_objects::object_handler::create_render_object;
use std::hint::black_box;

use std::fs;
use std::path::PathBuf;

fn avatar_benchmarks(c: &mut Criterion) {
    let scene_groups_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/raw_server_data/scenegroups");

    let mesh_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/raw_server_data/meshes");

    let texture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/textures");

    let scene_group_paths: Vec<PathBuf> = fs::read_dir(&scene_groups_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xml"))
        .collect();

    c.bench_function("scene_groups fs::read", |b| {
        b.iter(|| {
            let buffers: Vec<Vec<u8>> = scene_group_paths
                .iter()
                .map(|path| fs::read(black_box(path)).unwrap())
                .collect();

            black_box(buffers);
        });
    });

    let scene_group_xml: Vec<Vec<u8>> = scene_group_paths
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect();

    c.bench_function("SceneGroup::from_xml all", |b| {
        b.iter(|| {
            let scene_groups: Vec<SceneGroup> = scene_group_xml
                .iter()
                .map(|xml| SceneGroup::from_xml(black_box(xml)).unwrap())
                .collect();

            black_box(scene_groups);
        });
    });

    let scene_groups: Vec<SceneGroup> = scene_group_xml
        .iter()
        .map(|xml| SceneGroup::from_xml(xml).unwrap())
        .collect();

    let parts: Vec<_> = scene_groups
        .iter()
        .flat_map(|scene_group| scene_group.parts.iter())
        .collect();

    let mesh_paths: Vec<PathBuf> = parts
        .iter()
        .map(|part| mesh_dir.join(format!("{}.bin", part.sculpt.texture)))
        .collect();

    c.bench_function("mesh fs::read all", |b| {
        b.iter(|| {
            let meshes: Vec<Vec<u8>> = mesh_paths
                .iter()
                .map(|path| fs::read(black_box(path)).unwrap())
                .collect();

            black_box(meshes);
        });
    });

    let mesh_bytes: Vec<Vec<u8>> = mesh_paths
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect();

    c.bench_function("Mesh::from_bytes all", |b| {
        b.iter(|| {
            let meshes: Vec<Mesh> = mesh_bytes
                .iter()
                .map(|bytes| Mesh::from_bytes(black_box(bytes)).unwrap())
                .collect();

            black_box(meshes);
        });
    });

    let meshes: Vec<Mesh> = mesh_bytes
        .iter()
        .map(|bytes| Mesh::from_bytes(bytes).unwrap())
        .collect();

    let texture_paths: Vec<PathBuf> = parts
        .iter()
        .map(|part| texture_dir.join(format!("{}.png", part.shape.texture.texture_id)))
        .collect();

    c.bench_function("texture fs::read all", |b| {
        b.iter(|| {
            let textures: Vec<Vec<u8>> = texture_paths
                .iter()
                .map(|path| fs::read(black_box(path)).unwrap())
                .collect();

            black_box(textures);
        });
    });

    c.bench_function("create_render_object all", |b| {
        b.iter(|| {
            let render_objects: Vec<RenderObject> = parts
                .iter()
                .zip(meshes.iter())
                .zip(texture_paths.iter())
                .map(|((part, mesh), texture_path)| {
                    create_render_object(
                        black_box(mesh.clone()),
                        black_box(part.metadata.name.clone()),
                        black_box(texture_path),
                        part.sculpt.texture,
                    )
                    .unwrap()
                })
                .collect();

            black_box(render_objects);
        });
    });

    let render_objects: Vec<RenderObject> = parts
        .iter()
        .zip(meshes)
        .zip(texture_paths.iter())
        .map(|((part, mesh), texture_path)| {
            create_render_object(
                mesh,
                part.metadata.name.clone(),
                texture_path,
                part.sculpt.texture,
            )
            .unwrap()
        })
        .collect();

    c.bench_function("serde_json::to_vec all", |b| {
        b.iter(|| {
            black_box(serde_json::to_vec(black_box(&render_objects)).unwrap());
        });
    });
    let json = serde_json::to_vec(&render_objects).unwrap();
    let output_path = std::env::temp_dir().join("benthic_benchmark.json");

    c.bench_function("json fs::write all", |b| {
        b.iter(|| {
            fs::write(black_box(&output_path), black_box(&json)).unwrap();
        });
    });

    let generated_agent_dir = std::env::temp_dir().join("benthic_benchmark_agent");

    c.bench_function("fs::create_dir_all", |b| {
        b.iter(|| {
            fs::create_dir_all(black_box(&generated_agent_dir)).unwrap();
        });
    });
}

criterion_group!(benches, avatar_benchmarks);
criterion_main!(benches);
