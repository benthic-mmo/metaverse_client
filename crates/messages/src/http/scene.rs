use crate::{
    errors::ParseError,
    udp::core::object_update::{MaterialType, ObjectUpdate},
    utils::{
        item_metadata::{ItemMetadata, SaleType},
        object_types::ObjectType,
    },
};
use glam::{Vec3, Vec4, bool};
use quick_xml::{
    Reader,
    escape::unescape,
    events::{BytesText, Event},
};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use std::{
    str::{FromStr, from_utf8},
    time::SystemTime,
};
use uuid::Uuid;

use super::mesh::Mesh;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// The group of scenes received from the server, used for rendering meshes.
///
/// Scenes contain attributes for the meshes contained within them. They describe which meshes are
/// present, how they are textured, and how they are placed and parented in relation to each other.
/// This is the main structure for how multi-mesh objects are created in OpenSimulator. Upon upload
/// to the server, each multipart mesh is broken up into its component pieces, and given a unique
/// UUID. SceneGroups describe how those individual pieces should be reconstituted back into a
/// single mesh.
pub struct SceneGroup {
    /// The parts of the root object. These contain different meshes that make up the whole object.
    /// The root object is always at position 0 of the vec.
    pub parts: Vec<SceneObject>,
}
impl SceneGroup {
    /// Receive bytes from the server, and parse them as XML.
    ///
    /// Despite the majority of the project using things like LLSD, or LLSD-XML, this uses raw 2001 standard
    /// xml. XML is not a very good option, because of how hard it can be to iterate over data that
    /// can contain a variable number of entries, while having to read line by line.
    /// If this were JSON, life would be much better.
    pub fn from_xml(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut root_object = SceneObject::default();
        let mut children = Vec::new();
        let mut reader = Reader::from_reader(bytes);
        let mut buf = Vec::new();
        let mut path: Vec<String> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    path.push(from_utf8(e.name().as_ref())?.to_string());
                    // If we are at the part of the XML describing the other parts, create a new
                    // scene object that will be mutated by the read_until_scene_object_end
                    // function.
                    if path.ends_with(&[
                        "SceneObjectGroup".to_string(),
                        "OtherParts".to_string(),
                        "Part".to_string(),
                        "SceneObjectPart".to_string(),
                    ]) {
                        let mut obj = SceneObject::default();
                        SceneObject::read_scene_object(&mut reader, &mut obj, path.clone())?;
                        children.push(obj);
                        path.pop(); // pop "SceneObjectPart"
                    }
                }
                Event::End(_) => {
                    path.pop();
                }
                Event::Text(e) => {
                    let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
                    // If we are at the part of the XML describing the RootPart, mutate the
                    // root_object SceneOBject field by field.
                    if path_refs.starts_with(&["SceneObjectGroup", "RootPart", "SceneObjectPart"]) {
                        SceneObject::from_path_str(path_refs.clone(), e, &mut root_object, 3)?;
                    }
                }
                Event::Eof => break,
                _ => {}
            }

            buf.clear();
        }
        // set the root object to the first child.
        children.insert(0, root_object);
        Ok(SceneGroup { parts: children })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// An object within a scene
///
/// This contains information about the object, and how it relates to other objects within the
/// scene.
pub struct SceneObject {
    /// ID of the person who created the object
    pub creator_id: Uuid,
    /// The folder the object is located in
    pub folder_id: Uuid,
    /// A serial number that gets incrimented upon every change of the object.
    /// This is used to ensure the inventory is up to date.
    pub inventory_serial: i32,
    /// The name of the mesh
    pub name: String,
    /// A flag that determines if a prim passes on click events to prims behind it.
    /// This is useful for things like HUDs where things are visually in front, but should not
    /// block input to things behind.
    pub pass_touches: bool,
    /// A flag that determines if a prim passes collision events to the prim behind it.
    /// useful for things like dresses where you don't want to set the collision for the dress
    /// itself, but the body wearing it.
    pub pass_collisions: bool,
    /// X location of the agent in the world. Parsed from RegionHandle.
    pub region_x: u32,
    /// Y location of the agent of the world. Parsed from RegionHandle
    pub region_y: u32,
    /// A pin to prevent unauthorized script injections. Kind of like a password.
    /// This is very insecure and should be phased out.
    pub script_access_pin: u64,
    /// World position of the root. The true location of the object.
    pub group_position: Vec3,
    /// offset of the object. If it is the root, this is the offset from the group position. if it
    /// is a child, it's the offset from the root.
    pub offset_position: Vec3,
    /// Description of the object. Could be used for image descriptions for MUD based viewers.
    pub description: String,
    /// Deprecated commerce type
    pub category: String,
    /// Tint applied to the object when rendering
    pub color: Rgba<i32>,
    /// Label returned when the object is clicked
    pub touch_name: String,
    /// Index of the prim in the scene. 1 is the root.
    pub link_num: i32,
    /// UUID of the thing that most recently rezzed te object.
    pub rezzer_id: Uuid,
    /// The sound the object makes on collision
    pub collision_sound: Uuid,
    /// The volume of the collision sound
    pub collision_sound_volume: i32,
    /// Information about the Sculpt.
    /// Contains information about how the viewer will render the geometry of the object.
    pub sculpt: Sculpt,
    /// Information about sitting
    /// contains information about how the position and orientation of agent sitting.
    pub sit_data: SitData,
    /// Information about the item's metadata
    /// This contains things like the permissions, sale info, item and asset ID and etc.
    pub item_metadata: ItemMetadata,
    /// Information about the ObjectUpdate
    /// THis contains things like the primitive geometry, motion data, materials and etc
    pub object_update: ObjectUpdate,
    /// Used by vendors to populate pricetags
    pub pay_price: [i32; 5],
    /// Offset of the attachment relative to the avatar bone when worn
    /// allows repositioning of attachments without editing the root.
    pub attached_pos: Vec3,
    /// determines the effect of gravity on the object.
    /// 0 = normal gravity
    /// 1 = floating
    /// \>1 = rising
    pub buoyancy: i32,
    /// Constant force applied by physics each frame. Used for things like hovering platforms.
    pub force: Vec3,
    /// Constant torque applied by physics each frame. Used for things like spinners and wheels.
    pub torque: Vec3,
    /// Flag for determining if the object is a non-solid "Sensor volume" that triggers on
    /// collisions. Used for triggers without physical shape.
    pub volume_detect_active: bool,
    /// Controls collision hull optimization.
    pub physics_shape_type: PhysicsShapeType,
    /// Offset applied to the avatar's camera when seated on this prim. For things like changing
    /// camera angle when driving a car. Matches edit camera settings.
    pub camera_eye_offset: Vec3,
    /// "Look at" focus offset for the camera eye offset. For things like changing camera angle
    /// when driving a car.
    pub camera_at_offset: Vec3,
    /// Flag for preventing sounds from overlapping by queuing them instead. Used to prevent audio
    /// spam.
    pub sound_queueing: bool,
}

impl SceneObject {
    /// This is used for reading the bytes after the root object has been parsed.
    /// This is for handling the OtherParts section of the xml.
    fn read_scene_object<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        scene_object: &mut SceneObject,
        path_prefix: Vec<String>,
    ) -> Result<(), ParseError> {
        let mut buf = Vec::new();
        let mut path = path_prefix;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    path.push(from_utf8(e.name().as_ref())?.to_string());
                }
                Event::End(ref e) => {
                    let tag_bytes = e.name(); // get the raw name
                    let tag = from_utf8(tag_bytes.as_ref())?; // convert to &str
                    if let Some(last) = path.last()
                        && last == tag
                    {
                        path.pop();
                    }
                    // Exit if we've closed the original SceneObjectPart
                    if tag == "SceneObjectPart" {
                        break;
                    }

                    // Exit if we've closed the original SceneObjectPart
                    if tag == "SceneObjectPart"
                        && !path.ends_with(&[
                            "Part".to_string(),
                            "OtherParts".to_string(),
                            "SceneObjectGroup".to_string(),
                        ])
                    {
                        break;
                    }
                }
                Event::Text(e) => {
                    let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
                    SceneObject::from_path_str(path_refs, e, scene_object, 4)?;
                }
                Event::Eof => break,
                _ => {}
            }

            buf.clear();
        }
        Ok(())
    }
    /// from_path_str takes one path string, and parses the scene object value out of it.
    /// This takes a mutable scene object, and changes its default value to the value defined by
    /// the xml.
    pub fn from_path_str(
        path_str: Vec<&str>,
        e: BytesText<'_>,
        scene_object: &mut SceneObject,
        offset: usize,
    ) -> Result<(), ParseError> {
        let text = str::from_utf8(e.as_ref())?;
        let val = unescape(text)?.into_owned();
        match &path_str[offset..] {
            ["CreatorID", "UUID"] => {
                scene_object.creator_id = Uuid::parse_str(&val)?;
            }
            ["FolderId", "UUID"] => {
                scene_object.folder_id = Uuid::parse_str(&val)?;
            }
            ["InventorySerial"] => {
                scene_object.inventory_serial = val.parse::<i32>()?;
            }
            ["UUID", "UUID"] => {
                scene_object.object_update.full_id = Uuid::parse_str(&val)?;
            }
            ["LocalId"] => {
                scene_object.object_update.id = val.parse()?;
            }
            ["Name"] => {
                scene_object.name = val;
            }
            ["Material"] => {
                scene_object.object_update.material = MaterialType::from_bytes(&val.parse::<u8>()?);
            }
            ["PassTouches"] => {
                scene_object.pass_touches = val.parse::<bool>()?;
            }
            ["PassCollisions"] => {
                scene_object.pass_collisions = val.parse::<bool>()?;
            }
            ["RegionHandle"] => {
                let region_handle = val.parse::<u64>()?;
                scene_object.region_x = (region_handle >> 32) as u32;
                scene_object.region_y = (region_handle & 0xFFFF_FFFF) as u32;
            }
            ["ScriptAccessPin"] => {
                scene_object.script_access_pin = val.parse::<u64>()?;
            }

            ["GroupPosition", rest @ ..] => match rest {
                ["X"] => scene_object.group_position[0] = val.parse()?,
                ["Y"] => scene_object.group_position[1] = val.parse()?,
                ["Z"] => scene_object.group_position[2] = val.parse()?,
                _ => {}
            },
            ["OffsetPosition", rest @ ..] => match rest {
                ["X"] => scene_object.offset_position[0] = val.parse()?,
                ["Y"] => scene_object.offset_position[1] = val.parse()?,
                ["Z"] => scene_object.offset_position[2] = val.parse()?,
                _ => {}
            },
            ["RotationOffset", rest @ ..] => match rest {
                ["X"] => scene_object.object_update.motion_data.rotation[0] = val.parse()?,
                ["Y"] => scene_object.object_update.motion_data.rotation[1] = val.parse()?,
                ["Z"] => scene_object.object_update.motion_data.rotation[2] = val.parse()?,
                _ => {}
            },
            ["Velocity", rest @ ..] => match rest {
                ["X"] => scene_object.object_update.motion_data.velocity[0] = val.parse()?,
                ["Y"] => scene_object.object_update.motion_data.velocity[1] = val.parse()?,
                ["Z"] => scene_object.object_update.motion_data.velocity[2] = val.parse()?,
                _ => {}
            },
            ["AngularVelocity", rest @ ..] => match rest {
                ["X"] => {
                    scene_object.object_update.motion_data.angular_velocity[0] = val.parse()?
                }
                ["Y"] => {
                    scene_object.object_update.motion_data.angular_velocity[1] = val.parse()?
                }
                ["Z"] => {
                    scene_object.object_update.motion_data.angular_velocity[2] = val.parse()?
                }
                _ => {}
            },
            ["Acceleration", rest @ ..] => match rest {
                ["X"] => scene_object.object_update.motion_data.acceleration[0] = val.parse()?,
                ["Y"] => scene_object.object_update.motion_data.acceleration[1] = val.parse()?,
                ["Z"] => scene_object.object_update.motion_data.acceleration[2] = val.parse()?,
                _ => {}
            },
            ["Description"] => {
                scene_object.description = val;
            }
            ["Color", rest @ ..] => match rest {
                ["R"] => {
                    scene_object.color.r = val.parse::<i32>()?;
                }
                ["G"] => {
                    scene_object.color.g = val.parse::<i32>()?;
                }
                ["B"] => {
                    scene_object.color.b = val.parse::<i32>()?;
                }
                ["A"] => {
                    scene_object.color.a = val.parse::<i32>()?;
                }
                _ => {}
            },
            ["Text"] => {
                scene_object.object_update.text = val;
            }
            ["SitName"] => {
                scene_object.sit_data.sit_name = val;
            }
            ["TouchName"] => {
                scene_object.touch_name = val;
            }
            ["LinkNum"] => {
                scene_object.link_num = val.parse::<i32>()?;
            }
            ["ClickAction"] => {
                scene_object.object_update.click_action = val.parse()?;
            }
            ["Shape", rest @ ..] => match rest {
                ["ProfileCurve"] => {
                    scene_object.object_update.primitive_geometry.profile_curve =
                        val.parse::<u8>()?
                }
                ["TextureEntry"] => {
                    scene_object.object_update.texture_entry = val.as_bytes().to_vec()
                }
                ["ExtraParams"] => {
                    scene_object.object_update.extra_params = val.as_bytes().to_vec()
                }
                ["PathBegin"] => {
                    scene_object.object_update.primitive_geometry.path_begin = val.parse::<u16>()?
                }
                ["PathCurve"] => {
                    scene_object.object_update.primitive_geometry.path_curve = val.parse::<u8>()?
                }
                ["PathEnd"] => {
                    scene_object.object_update.primitive_geometry.path_end = val.parse::<u16>()?
                }
                ["PathRadiusOffset"] => {
                    scene_object
                        .object_update
                        .primitive_geometry
                        .path_radius_offset = val.parse::<i8>()?
                }
                ["PathRevolutions"] => {
                    scene_object
                        .object_update
                        .primitive_geometry
                        .path_revolutions = val.parse::<u8>()?
                }
                ["PathScaleX"] => {
                    scene_object.object_update.primitive_geometry.path_scale_x =
                        val.parse::<u8>()?
                }
                ["PathScaleY"] => {
                    scene_object.object_update.primitive_geometry.path_scale_y =
                        val.parse::<u8>()?
                }
                ["PathShearX"] => {
                    scene_object.object_update.primitive_geometry.path_shear_x =
                        val.parse::<u8>()?
                }
                ["PathShearY"] => {
                    scene_object.object_update.primitive_geometry.path_shear_y =
                        val.parse::<u8>()?
                }
                ["PathSkew"] => {
                    scene_object.object_update.primitive_geometry.path_skew = val.parse::<i8>()?
                }
                ["PathTaperX"] => {
                    scene_object.object_update.primitive_geometry.path_taper_x =
                        val.parse::<i8>()?
                }
                ["PathTaperY"] => {
                    scene_object.object_update.primitive_geometry.path_taper_y =
                        val.parse::<i8>()?
                }
                ["PathTwist"] => {
                    scene_object.object_update.primitive_geometry.path_twist_end =
                        val.parse::<i8>()?
                }
                ["PathTwistBegin"] => {
                    scene_object
                        .object_update
                        .primitive_geometry
                        .path_twist_begin = val.parse::<i8>()?
                }
                ["PCode"] => {
                    scene_object.object_update.pcode = ObjectType::from_bytes(&val.parse::<u8>()?);
                }
                ["ProfileBegin"] => {
                    scene_object.object_update.primitive_geometry.profile_begin =
                        val.parse::<u16>()?
                }
                ["ProfileEnd"] => {
                    scene_object.object_update.primitive_geometry.profile_end =
                        val.parse::<u16>()?
                }
                ["ProfileHollow"] => {
                    scene_object.object_update.primitive_geometry.profile_hollow =
                        val.parse::<f32>()?
                }
                ["ProfileShape"] => {
                    scene_object.object_update.primitive_geometry.profile_shape = Some(val)
                }
                ["HollowShape"] => {
                    scene_object.object_update.primitive_geometry.hollow_shape = Some(val)
                }

                ["State"] => scene_object.sculpt.state = val.parse::<i32>()?,
                ["LastAttachPoint"] => {
                    scene_object.sculpt.last_attach_point = val.parse::<i32>()?
                }
                ["SculptTexture", "UUID"] => scene_object.sculpt.texture = Uuid::parse_str(&val)?,
                ["SculptType"] => {
                    scene_object.sculpt.sculpt_type = SculptType::from_bytes(val.parse::<i32>()?)
                }
                ["SculptEntry"] => scene_object.sculpt.entry = val.parse::<bool>()?,
                ["FlexiSoftness"] => scene_object.sculpt.flex.softness = val.parse::<i32>()?,
                ["FlexiTension"] => scene_object.sculpt.flex.tension = val.parse::<i32>()?,
                ["FlexiDrag"] => scene_object.sculpt.flex.drag = val.parse::<i32>()?,
                ["FlexiGravity"] => scene_object.sculpt.flex.gravity = val.parse::<i32>()?,
                ["FlexiWind"] => scene_object.sculpt.flex.wind = val.parse::<i32>()?,

                ["FlexiForceX"] => scene_object.sculpt.flex.force[0] = val.parse::<f32>()?,
                ["FlexiForceY"] => scene_object.sculpt.flex.force[1] = val.parse::<f32>()?,
                ["FlexiForceZ"] => scene_object.sculpt.flex.force[2] = val.parse::<f32>()?,
                ["LightColorR"] => scene_object.sculpt.light.color.r = val.parse::<i32>()?,
                ["LightColorG"] => scene_object.sculpt.light.color.g = val.parse::<i32>()?,
                ["LightColorB"] => scene_object.sculpt.light.color.b = val.parse::<i32>()?,
                ["LightColorA"] => scene_object.sculpt.light.color.a = val.parse::<i32>()?,
                ["LightRadius"] => scene_object.sculpt.light.radius = val.parse::<i32>()?,
                ["LightCutoff"] => scene_object.sculpt.light.cutoff = val.parse::<i32>()?,
                ["LightFalloff"] => scene_object.sculpt.light.falloff = val.parse::<i32>()?,
                ["LightIntensity"] => scene_object.sculpt.light.intensity = val.parse::<i32>()?,
                ["FlexiEntry"] => scene_object.sculpt.flex.entry = val.parse::<bool>()?,
                ["LightEntry"] => scene_object.sculpt.light.entry = val.parse::<bool>()?,
                _ => {}
            },

            ["Scale", rest @ ..] => match rest {
                ["X"] => scene_object.object_update.scale[0] = val.parse()?,
                ["Y"] => scene_object.object_update.scale[1] = val.parse()?,
                ["Z"] => scene_object.object_update.scale[2] = val.parse()?,
                _ => {}
            },
            ["SitTargetOrientation", rest @ ..] => match rest {
                ["X"] => scene_object.sit_data.orientation[0] = val.parse()?,
                ["Y"] => scene_object.sit_data.orientation[1] = val.parse()?,
                ["Z"] => scene_object.sit_data.orientation[2] = val.parse()?,
                _ => {}
            },
            ["SitTargetOrientationLL", rest @ ..] => match rest {
                ["X"] => scene_object.sit_data.orientation_ll[0] = val.parse()?,
                ["Y"] => scene_object.sit_data.orientation_ll[1] = val.parse()?,
                ["Z"] => scene_object.sit_data.orientation_ll[2] = val.parse()?,
                _ => {}
            },
            ["SitTargetPosition", rest @ ..] => match rest {
                ["X"] => scene_object.sit_data.position[0] = val.parse()?,
                ["Y"] => scene_object.sit_data.position[1] = val.parse()?,
                ["Z"] => scene_object.sit_data.position[2] = val.parse()?,
                _ => {}
            },
            ["SitTargetPositionLL", rest @ ..] => match rest {
                ["X"] => scene_object.sit_data.position_ll[0] = val.parse()?,
                ["Y"] => scene_object.sit_data.position_ll[1] = val.parse()?,
                ["Z"] => scene_object.sit_data.position_ll[2] = val.parse()?,
                _ => {}
            },
            ["ParentID"] => {
                scene_object.item_metadata.parent_id_local = if val == "0" {
                    Some(0)
                } else {
                    Some(u32::from_str(&val)?)
                };
            }
            ["CreationDate"] => {
                scene_object.item_metadata.created_at =
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(val.parse::<u64>()?)
            }
            ["Category"] => scene_object.category = val,
            ["SalePrice"] => scene_object.item_metadata.sale_info.price = val.parse::<i32>()?,
            ["ObjectSaleType"] => {
                scene_object.item_metadata.sale_info.sale_type = SaleType::from_string(&val)
            }

            ["OwnershipCost"] => {
                scene_object.item_metadata.sale_info.ownership_cost = Some(val.parse::<i32>()?)
            }
            ["GroupID", "UUID"] => {
                scene_object.item_metadata.permissions.group_id = Uuid::from_str(&val)?
            }
            ["OwnerID", "UUID"] => {
                scene_object.item_metadata.permissions.owner_id = Uuid::from_str(&val)?
            }
            ["LastOwnerID", "UUID"] => {
                scene_object.item_metadata.permissions.last_owner_id = Some(Uuid::from_str(&val)?)
            }
            ["RezzerID", "UUID"] => scene_object.rezzer_id = Uuid::from_str(&val)?,
            ["BaseMask"] => {
                scene_object.item_metadata.permissions.base_mask = val.parse::<i32>()?
            }
            ["OwnerMask"] => {
                scene_object.item_metadata.permissions.owner_mask = val.parse::<i32>()?
            }
            ["Groupmask"] => {
                scene_object.item_metadata.permissions.group_mask = val.parse::<i32>()?
            }
            ["EveryoneMask"] => {
                scene_object.item_metadata.permissions.everyone_mask = val.parse::<i32>()?
            }
            ["NextOwnerMask"] => {
                scene_object.item_metadata.permissions.next_owner_mask = val.parse::<i32>()?
            }
            ["Flags"] => {
                if val == "None" {
                    scene_object.item_metadata.flags = 0
                } else {
                    scene_object.item_metadata.flags = val.parse::<i32>()?
                }
            }
            ["CollisionSound", "UUID"] => scene_object.collision_sound = Uuid::from_str(&val)?,
            ["CollisionSoundVolume"] => scene_object.collision_sound_volume = val.parse::<i32>()?,
            ["AttachedPos", rest @ ..] => match rest {
                ["X"] => scene_object.attached_pos[0] = val.parse()?,
                ["Y"] => scene_object.attached_pos[1] = val.parse()?,
                ["Z"] => scene_object.attached_pos[2] = val.parse()?,
                _ => {}
            },
            ["Textureanimation"] => scene_object.object_update.texture_anim = val.into_bytes(),
            ["ParticleSystem"] => {
                scene_object.object_update.particle_system_block = val.into_bytes()
            }
            ["PayPrice0"] => scene_object.pay_price[0] = val.parse::<i32>()?,
            ["PayPrice1"] => scene_object.pay_price[1] = val.parse::<i32>()?,
            ["PayPrice2"] => scene_object.pay_price[2] = val.parse::<i32>()?,
            ["PayPrice3"] => scene_object.pay_price[3] = val.parse::<i32>()?,
            ["PayPrice4"] => scene_object.pay_price[4] = val.parse::<i32>()?,
            ["Buoyancy"] => scene_object.buoyancy = val.parse::<i32>()?,
            ["Force", rest @ ..] => match rest {
                ["X"] => scene_object.force[0] = val.parse()?,
                ["Y"] => scene_object.force[1] = val.parse()?,
                ["Z"] => scene_object.force[2] = val.parse()?,
                _ => {}
            },
            ["Torque", rest @ ..] => match rest {
                ["x"] => scene_object.torque[0] = val.parse()?,
                ["y"] => scene_object.torque[1] = val.parse()?,
                ["z"] => scene_object.torque[2] = val.parse()?,
                _ => {}
            },
            ["VolumeDetectActive"] => scene_object.volume_detect_active = val.parse::<bool>()?,
            ["PhysicsShapetype"] => {
                scene_object.physics_shape_type = PhysicsShapeType::from_bytes(val.parse::<i32>()?)
            }
            ["CameraEyeOffset", rest @ ..] => match rest {
                ["x"] => scene_object.camera_eye_offset[0] = val.parse()?,
                ["y"] => scene_object.camera_eye_offset[1] = val.parse()?,
                ["z"] => scene_object.camera_eye_offset[2] = val.parse()?,
                _ => {}
            },
            ["CameraAtOffset", rest @ ..] => match rest {
                ["x"] => scene_object.camera_at_offset[0] = val.parse()?,
                ["y"] => scene_object.camera_at_offset[1] = val.parse()?,
                ["z"] => scene_object.camera_at_offset[2] = val.parse()?,
                _ => {}
            },
            ["SoundID", "UUID"] => {
                scene_object.object_update.sound.sound_id = Uuid::from_str(&val)?
            }
            ["SoundGain"] => scene_object.object_update.sound.gain = val.parse::<f32>()?,
            ["SoundFlags"] => scene_object.object_update.sound.flags = val.parse::<u8>()?,
            ["SoundRadius"] => scene_object.object_update.sound.radius = val.parse::<f32>()?,
            ["SoundQueueing"] => scene_object.sound_queueing = val.parse::<bool>()?,
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Sculpt information
/// <https://wiki.secondlife.com/wiki/Sculpted_Prims:_FAQ>
///
/// Sculpts were used before meshes became widespread. They were created using SculptTextures,
/// which was a standard RGB texture where the R,G and B values were mapped onto values in x,y,z
/// space. This allowed low-bandwith distribution for meshes, and easy animation. Flash animations
/// could easily generate animations, by playing the sculpt textures like a traditional animation,
/// and compression and decompression were very straightforward.
///
/// This has been almost completely replaced by modern meshes, but the name "Sculpt" remains the
/// standard.
pub struct Sculpt {
    /// The UUID of the "sculpt texture".
    /// This used to contain the UUID of the SculptTexture image, but now contains the UUID of the
    /// mesh object.
    pub texture: Uuid,
    /// Tells the engine what base to use when deforming based on the SculptTexture.
    /// For example, the SculptTexture applied to a sphere would deform that sphere, or a
    /// SculptTexture applied to a plane would deform differently.
    /// Mostly legacy now. Always set to 5 when displaying meshes.
    pub sculpt_type: SculptType,
    /// Used to determine if the object can be entered and walked through.
    pub entry: bool,
    /// Information about the flex attributes
    /// Contains information about how the object bends, sways and moves using gravity, tension and
    /// wind.
    pub flex: Flex,
    /// Information about the light attributes.
    /// Contains information about how the object emits lights.
    pub light: Light,
    /// Describes the script state of the object. Default, on, off, etc. Helps resume script
    /// execution after duplication or region restart.
    pub state: i32,
    /// The last point the object was attached to. Helps reattach items to the same place when
    /// logging in or putting on again.
    pub last_attach_point: i32,
    /// The optional downloaded mesh, retrieved from the AssetServer.
    /// Will be empty until the mesh data is retrieved from the server.
    pub mesh: Option<Mesh>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Flex information
///
/// Used to describe how physics forces act on a non-rigid object.
pub struct Flex {
    /// Enables flexible behavior. If false, the object is rigid.
    pub entry: bool,
    /// Controls the number of segements in the flexible object. Higher = more bendy.
    pub softness: i32,
    /// A spring-like force that pulls the prim back to its original position.
    /// higher = tighter
    pub tension: i32,
    /// Damping effect. High values reduce fast motion.
    pub drag: i32,
    /// Gravity's downward force on the object
    pub gravity: i32,
    /// the effect wind has on the object
    pub wind: i32,
    /// A custom force applied to the object
    pub force: Vec4,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Information about how the object can be sat on.
pub struct SitData {
    /// Custom name that is displayed when the user right-clicks to sit.
    /// Could be things like "sit", "pose", "lounge" etc
    pub sit_name: String,
    /// How to rotate the avatar that sits down
    pub orientation: Vec3,
    /// Undocumented. Maybe legacy rotation vector
    pub orientation_ll: Vec3,
    /// XYZ position the avatar should be placed when sitting.
    pub position: Vec3,
    /// Undocumented. Maybe legacy position vector.
    pub position_ll: Vec3,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Information about how the object emits light
pub struct Light {
    /// Enables glow. If false, the object doesn't glow.
    pub entry: bool,
    /// The color of the glow
    pub color: Rgba<i32>,
    /// how far the light reaches from the source
    pub radius: i32,
    /// Cone cutoff angle for spotlights. 0 = point light, 180 is a full sphere.
    pub cutoff: i32,
    /// Brightness of the light
    pub intensity: i32,
    /// How quickly the light intensity decreases with distance
    pub falloff: i32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// The types used for physics collision
pub enum PhysicsShapeType {
    /// Primitive collision
    Prim,
    /// The smallest convex shape that contains all the points of the 3d mesh.
    /// Kind of like shrinkwrapping to find the collision points of the mesh. Does not handle
    /// collision for inclusions.
    ConvexHull,
    /// No collision
    None,

    #[default]
    /// Unknown
    Unknown,
}
impl PhysicsShapeType {
    fn from_bytes(bytes: i32) -> Self {
        match bytes {
            0 => Self::Prim,
            1 => Self::ConvexHull,
            2 => Self::None,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Used for legacy compatability with SculptTextures.
/// describes the different basic shapes the sculpttexture can deform.
pub enum SculptType {
    /// The sculpt texture deforms a sphere
    Sphere,
    /// The sculpt texture deforms a torus
    Torus,
    /// The sculpt texture deforms a plane
    Plane,
    /// The sculpt texture deforms a cylinder
    Cylinder,
    /// This describes a mesh. which is the primary supported type.
    Mesh,

    #[default]
    /// Unknown
    Unknown,
}
impl SculptType {
    fn from_bytes(bytes: i32) -> Self {
        match bytes {
            1 => Self::Sphere,
            2 => Self::Torus,
            3 => Self::Plane,
            4 => Self::Cylinder,
            5 => Self::Mesh,
            _ => Self::Unknown,
        }
    }
}
