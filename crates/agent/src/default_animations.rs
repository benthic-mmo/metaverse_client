use std::path::PathBuf;
use uuid::Uuid;

pub struct AnimData {
    pub uuid: Uuid,
    pub path: PathBuf,
}

macro_rules! define_animations {
    (
        $( $name:ident => ($uuid:expr, $path:expr) ),* $(,)?
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub enum DefaultAnimation {
            $( $name ),*
        }

        impl DefaultAnimation {
            pub fn data(&self) -> AnimData {
                match self {
                    $(
                        DefaultAnimation::$name => AnimData {
                            uuid: Uuid::parse_str($uuid).unwrap(),
                            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                .join("assets")
                                .join("animations")
                                .join($path),
                        }
                    ),*
                }
            }

            pub fn from_uuid(uuid: &Uuid) -> Option<Self> {
                $(
                    if DefaultAnimation::$name.data().uuid == *uuid {
                        return Some(DefaultAnimation::$name);
                    }
                )*
                None
            }

            pub fn path_for_uuid(uuid: &Uuid) -> Option<PathBuf> {
                Self::from_uuid(uuid).map(|anim| anim.data().path)
            }
        }
    };
}

define_animations! {
    Stand => ("2408fe9e-df1d-1d7d-f4ff-1384fa7b350f", "kitty_walk_test.glb"),
}
