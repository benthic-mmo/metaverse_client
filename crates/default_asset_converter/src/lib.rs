use std::path::PathBuf;

#[cfg(feature = "animations")]
#[allow(unused)]
use strum_macros::Display;

#[cfg(feature = "animations")]
#[allow(unused)]
use strum_macros::EnumString;

#[cfg(feature = "animations")]
#[allow(unused)]
use uuid::Uuid;

pub mod generated {
    use benthic_protocol::skeleton::Skeleton;
    use once_cell::sync::Lazy;

    pub static DEFAULT_SKELETON: Lazy<Skeleton> = Lazy::new(|| {
        serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/default_skeleton.json"
        )))
        .expect("Failed to deserialize default skeleton")
    });
}

pub fn generated_asset_path() -> PathBuf {
    PathBuf::from(env!("OUT_DIR"))
}

pub fn generated_animation_path() -> PathBuf {
    PathBuf::from(env!("OUT_DIR")).join("Animations")
}

pub fn default_texture_path() -> PathBuf {
    PathBuf::from(concat!(env!("OUT_DIR"), "/default.png"))
}

#[cfg(feature = "animations")]
#[allow(unused)]
macro_rules! define_animations {
    (
        $( $name:ident => $uuid:expr ),* $(,)?
    ) => {

        pub mod animations {
            $(
                pub mod $name {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/",
                        stringify!($name),
                        ".json"
                    ));
                }
            )*
        }
        pub fn joints(&self) -> &'static [Joint] {
            match self {
                $(
                    Self::$name => &crate::generated::animations::$name::JOINTS,
                )*
            }
        }
    };
}
