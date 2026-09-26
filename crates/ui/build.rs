use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    // prevent this copying on docs.rs, which causes the build to fail
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }
    let source_assets = benthic_default_asset_converter::default_assets();

    let source_textures = source_assets.join("Textures");
    let source_shaders = source_assets.join("Shaders");
    let source_cubemaps = source_assets.join("Cubemaps");

    let target_assets = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
    let target_textures = target_assets.join("Textures");
    let target_shaders = target_assets.join("Shaders");
    let target_cubemaps = target_assets.join("Cubemaps");

    // this moves the default assets into Bevy's asset folder.
    copy_dir(&source_shaders, &target_shaders);
    copy_dir(&source_textures, &target_textures);
    copy_dir(&source_cubemaps, &target_cubemaps);
}

fn copy_dir(src: &Path, dst: &Path) {
    if !dst.exists() {
        fs::create_dir_all(dst).unwrap();
    }

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir(&path, &dest_path);
        } else {
            fs::copy(&path, &dest_path).unwrap();
        }
    }
}
