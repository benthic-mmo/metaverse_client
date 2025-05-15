#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The types of all objects in the simulator. Also contains types for folders.
pub enum ObjectType {
    /// A bodypart
    Bodypart,
    /// Clothing
    Clothing,
    /// A primitive object. (cube, spehere, torus, etc)
    Prim,
    /// An avatar belonging to a user
    Avatar,
    /// A grass object
    Grass,
    /// A particle system object
    ParticleSystem,
    /// A tree
    Tree,
    /// An improved, more modern tree
    NewTree,
    /// An animation object
    Animation,
    /// A Calling Card. Effectively a link to someone's profile.  
    CallingCard,
    /// A landmark obect. A link to a place on the grid.
    Landmark,
    /// A gesture. This is an object that can include animations, sounds and chat messages.
    Gesture,
    /// A notecard. Contains text.
    Notecard,
    /// A default object. Either a primitive or several linked primitives.
    Object,
    /// A script object.
    Script,
    /// A sound.
    Sound,
    /// A texture.
    Texture,
    /// Objects to be deleted.
    Trash,
    /// A material. Stone, metal glass wood flesh plastic and rubber.
    Material,

    /// Photo Album type. Used by the folders for the screenshot folder.
    PhotoAlbum,
    /// Lost and found type. Used by the folders for items that have lost their parent.
    LostAndFound,
    /// Favorite type. Used by the folders to group favorites
    Favorite,
    /// Current Outfit type. Used by the folders to store the user's current outfit.
    CurrentOutfit,
    /// Setting type. Used by the folders to store settings.
    Setting,
    /// my outfit type. Used by the folders to store custom outfits.
    MyOutfit,
    /// received item type. Used by the folders to store items that were received by other users.
    ReceivedItem,

    /// unknown type
    Unknown,
}

impl ObjectType {
    /// Convert the object to its string representation. Useful for retrieving from the capability
    /// endpoint.
    pub fn to_string(&self) -> String {
        match self {
            ObjectType::Texture => "texture".to_string(),
            ObjectType::Sound => "sound".to_string(),
            ObjectType::CallingCard => "calling_card".to_string(),
            ObjectType::Landmark => "landmark".to_string(),
            ObjectType::Clothing => "clothing".to_string(),
            ObjectType::Object => "object".to_string(),
            ObjectType::Notecard => "notecard".to_string(),
            ObjectType::Prim => "prim".to_string(),
            ObjectType::Script => "script".to_string(),
            ObjectType::Bodypart => "bodypart".to_string(),
            ObjectType::Trash => "trash".to_string(),
            ObjectType::PhotoAlbum => "photo_album".to_string(),
            ObjectType::LostAndFound => "lost_and_found".to_string(),
            ObjectType::Animation => "animation".to_string(),
            ObjectType::Gesture => "gesture".to_string(),
            ObjectType::Favorite => "favorite".to_string(),
            ObjectType::CurrentOutfit => "current_outfit".to_string(),
            ObjectType::Avatar => "avatar".to_string(),
            ObjectType::MyOutfit => "my_outfit".to_string(),
            ObjectType::ReceivedItem => "received_item".to_string(),
            ObjectType::Setting => "setting".to_string(),
            ObjectType::Material => "material".to_string(),
            ObjectType::Grass => "grass".to_string(),
            ObjectType::NewTree => "new_tree".to_string(),
            ObjectType::ParticleSystem => "particle_system".to_string(),
            ObjectType::Tree => "tree".to_string(),
            ObjectType::Unknown => "unknown".to_string(),
        }
    }
    /// Maps the byte values of ObjectTypes to their correct data type
    pub fn to_bytes(&self) -> u8 {
        match self {
            ObjectType::Texture => 0,
            ObjectType::Sound => 1,
            ObjectType::CallingCard => 2,
            ObjectType::Landmark => 3,
            ObjectType::Clothing => 5,
            ObjectType::Object => 6,
            ObjectType::Notecard => 7,
            ObjectType::Prim => 9,
            ObjectType::Script => 10,
            ObjectType::Bodypart => 13,
            ObjectType::Trash => 14,
            ObjectType::PhotoAlbum => 15,
            ObjectType::LostAndFound => 16,
            ObjectType::Animation => 20,
            ObjectType::Gesture => 21,
            ObjectType::Favorite => 23,
            ObjectType::CurrentOutfit => 46,
            ObjectType::Avatar => 47,
            ObjectType::MyOutfit => 48,
            ObjectType::ReceivedItem => 50,
            ObjectType::Setting => 56,
            ObjectType::Material => 57,
            ObjectType::Grass => 95,
            ObjectType::NewTree => 111,
            ObjectType::ParticleSystem => 143,
            ObjectType::Tree => 255,
            ObjectType::Unknown => 99,
        }
    }

    /// convert bytes to their ObjectType representation
    pub fn from_bytes(bytes: &u8) -> Self {
        match bytes {
            0 => ObjectType::Texture,
            1 => ObjectType::Sound,
            2 => ObjectType::CallingCard,
            3 => ObjectType::Landmark,
            5 => ObjectType::Clothing,
            6 => ObjectType::Object,
            7 => ObjectType::Notecard,
            9 => ObjectType::Prim,
            10 => ObjectType::Script,
            13 => ObjectType::Bodypart,
            14 => ObjectType::Trash,
            15 => ObjectType::PhotoAlbum,
            16 => ObjectType::LostAndFound,
            20 => ObjectType::Animation,
            21 => ObjectType::Gesture,
            23 => ObjectType::Favorite,
            46 => ObjectType::CurrentOutfit,
            47 => ObjectType::Avatar,
            48 => ObjectType::MyOutfit,
            50 => ObjectType::ReceivedItem,
            56 => ObjectType::Setting,
            57 => ObjectType::Material,
            95 => ObjectType::Grass,
            111 => ObjectType::NewTree,
            143 => ObjectType::ParticleSystem,
            255 => ObjectType::Tree,
            _ => ObjectType::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Types for wearables. Used to determine what article of clothing they are.
pub enum WearableType {
    /// Shape of the user. Contains body dimensions.
    Shape,
    /// Skin of the user. The texture applied to their body
    Skin,
    /// Eyes of the user. The texture applied to their eyes.
    Eyes,

    /// Shirt
    Shirt,
    /// Pants
    Pants,
    /// Shoes
    Shoes,
    /// Socks
    Socks,
    /// Jacket, that goes over the shirt
    Jacket,
    /// Gloves
    Gloves,
    /// Undershirt, that goes under the shirt
    Undershirt,
    /// Underpants, goes under the pants
    Underpants,
    /// Skirt
    Skirt,
    /// Unknown
    Unknown,
}
impl WearableType {
    /// Used to determine the base type of the wearable. Used when retrieving from the ViewerAsset
    /// endpoint.
    pub fn category(&self) -> ObjectType {
        match self {
            WearableType::Shape | WearableType::Skin | WearableType::Eyes => ObjectType::Bodypart,
            WearableType::Shirt
            | WearableType::Pants
            | WearableType::Shoes
            | WearableType::Socks
            | WearableType::Jacket
            | WearableType::Gloves
            | WearableType::Undershirt
            | WearableType::Underpants
            | WearableType::Skirt => ObjectType::Clothing,
            WearableType::Unknown => ObjectType::Unknown,
        }
    }
    /// convert from bytes to the wearable type
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            0 => WearableType::Shape,
            1 => WearableType::Skin,
            3 => WearableType::Eyes,

            4 => WearableType::Shirt,
            5 => WearableType::Pants,
            6 => WearableType::Shoes,
            7 => WearableType::Socks,
            8 => WearableType::Jacket,
            9 => WearableType::Gloves,
            10 => WearableType::Undershirt,
            11 => WearableType::Underpants,
            12 => WearableType::Skirt,
            _ => WearableType::Unknown,
        }
    }
    /// convert from type to bytes
    pub fn to_bytes(&self) -> u8 {
        match self {
            WearableType::Shape => 0,
            WearableType::Skin => 1,
            WearableType::Eyes => 3,

            WearableType::Shirt => 4,
            WearableType::Pants => 5,
            WearableType::Shoes => 6,
            WearableType::Socks => 7,
            WearableType::Jacket => 8,
            WearableType::Gloves => 9,
            WearableType::Undershirt => 10,
            WearableType::Underpants => 11,
            WearableType::Skirt => 12,
            WearableType::Unknown => 99,
        }
    }
}
