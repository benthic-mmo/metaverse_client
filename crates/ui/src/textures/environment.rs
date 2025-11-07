use bevy::{
    asset::{Asset, Handle},
    color::LinearRgba,
    image::Image,
    pbr::Material,
    reflect::TypePath,
    render::{alpha::AlphaMode, render_resource::AsBindGroup},
    shader::ShaderRef,
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/terrain.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HeightMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Material for HeightMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
