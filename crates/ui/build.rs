use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let source_textures = benthic_default_assets::textures();
    let source_shaders = benthic_default_assets::shaders();

    let target_assets = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
    let target_textures = target_assets.join("textures");
    let target_shaders = target_assets.join("shaders");

    copy_dir(&source_shaders, &target_shaders);
    copy_dir(&source_textures, &target_textures);
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
