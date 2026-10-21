use strum_macros::{Display, EnumString};
use uuid::Uuid;

/// This generates the name to UUID mapping for default animations
macro_rules! define_animations {
    (
        $(
            $(#[$meta:meta])*
            $name:ident => $uuid:expr
        ),* $(,)?
    ) => {
        #[derive(Debug, Copy, Clone, Display, EnumString, PartialEq, Eq, Hash)]
        /// Enum containing all default animations
        pub enum DefaultAnimation {
            $(
                $(#[$meta])*
                $name
            ),*
        }

        impl DefaultAnimation {
            /// returns the UUID of the default animation
            pub fn uuid(&self) -> Uuid {
                match self {
                    $(
                        DefaultAnimation::$name => Uuid::parse_str($uuid).unwrap(),
                    )*
                }
            }

            /// Retrieves the animation's enum type from the UUID
            pub fn from_uuid(uuid: &Uuid) -> Option<Self> {
                $(
                    if Uuid::parse_str($uuid).unwrap() == *uuid {
                        return Some(DefaultAnimation::$name);
                    }
                )*
                None
            }

            /// Returns a DefaultAnimation object from a string
            pub fn from_string(name: &str) -> Option<Self> {
                name.parse().ok()
            }
        }
    };
}

// Map default animations to their UUID representation.
// This uses the macro to simplify maintaining the mapping.
define_animations! {
    /// Maps the UUID for the default stand animation
    Stand => "2408fe9e-df1d-1d7d-f4ff-1384fa7b350f",
    /// Maps the UUID for the default dance animation
    Dance => "2408fe9e-df1d-1d7d-f4ff-1384fa7b350a"
}
