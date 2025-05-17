use std::{
    str::{FromStr, from_utf8},
    time::SystemTime,
};

use glam::{bool, Vec3, Vec4};
use quick_xml::{Reader, events::Event};
use rgb::Rgba;
use uuid::Uuid;

use crate::{
    core::object_update::{MaterialType, ObjectUpdate},
    utils::object_types::ObjectType,
};

use super::item_types::{Item, SaleType};

#[derive(Debug, Default)]
pub struct SceneObject {
   pub creator_id: Uuid,
   pub folder_id: Uuid,
   pub inventory_serial: i32,
   pub name: String,
   pub pass_touches: bool,
   pub pass_collisions: bool,
   pub region_handle: u64,
   pub script_access_pin: u64,
   pub group_position: Vec3,
   pub description: String,
   pub category: String,
   pub color: Rgba<i32>,
   pub touch_name: String,
   pub link_num: i32,
   pub rezzer_id: Uuid,
   pub collision_sound: Uuid,
   pub collision_sound_volume: i32,
   pub sculpt: Sculpt,
   pub sit_data: SitData,
   pub item_data: Item,
   pub object_update: ObjectUpdate,
   pub pay_price: [i32; 5],
   pub attached_pos: Vec3,
   pub buoyancy: i32,
   pub force: Vec3,
   pub torque: Vec3,
   pub volume_detect_active: bool,
   pub physics_shape_type: i32,
   pub camera_eye_offset: Vec3,
   pub camera_at_offset: Vec3,
   pub sound_queueing: bool,
}

#[derive(Debug, Default)]
pub struct Sculpt {
   pub texture: Uuid,
   pub sculpt_type: i32,
   pub entry: bool,
   pub flex: Flex,
   pub light: Light,
   pub state: i32,
   pub last_attach_point: i32,
}

#[derive(Debug, Default)]
pub struct Flex {
   pub entry: bool,
   pub softness: i32,
   pub tension: i32,
   pub drag: i32,
   pub gravity: i32,
   pub wind: i32,
   pub force: Vec4,
}

#[derive(Debug, Default)]
pub struct SitData {
   pub sit_name: String,
   pub orientation: Vec3,
   pub orientation_ll: Vec3,
   pub position: Vec3,
   pub position_ll: Vec3,
}

#[derive(Debug, Default)]
pub struct Light {
   pub entry: bool,
   pub color: Rgba<i32>,
   pub radius: i32,
   pub cutoff: i32,
   pub intensity: i32,
   pub falloff: i32,
}

/// An ObjectUpdate object can also be retrieved from the ?object_id ViewerAsset endpoint when
/// downloading meshes and shapes. This comes in as w2 2001 xml.
/// This contains the information for retrieving
impl SceneObject {
    pub fn from_xml(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut scene_object = SceneObject::default();
        let mut reader = Reader::from_reader(bytes);
        let mut buf = Vec::new();
        let mut path: Vec<String> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    path.push(from_utf8(e.name().as_ref())?.to_string());
                }
                Event::End(_) => {
                    path.pop();
                }
                Event::Text(e) => {
                    let val = e.unescape()?.into_owned();
                    let path_str: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                    if path_str.starts_with(&["SceneObjectGroup", "RootPart", "SceneObjectPart"]) {
                        match &path_str[3..] {
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
                                scene_object.object_update.material =
                                    MaterialType::from_bytes(&val.parse::<u8>()?);
                            }
                            ["PassTouches"] => {
                                scene_object.pass_touches = val.parse::<bool>()?;
                            }
                            ["PassCollisions"] => {
                                scene_object.pass_collisions = val.parse::<bool>()?;
                            }
                            ["RegionHandle"] => {
                                scene_object.region_handle = val.parse::<u64>()?;
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
                                ["X"] => {
                                    scene_object.object_update.object_data.position[0] =
                                        val.parse()?
                                }
                                ["Y"] => {
                                    scene_object.object_update.object_data.position[1] =
                                        val.parse()?
                                }
                                ["Z"] => {
                                    scene_object.object_update.object_data.position[2] =
                                        val.parse()?
                                }
                                _ => {}
                            },
                            ["RotationOffset", rest @ ..] => match rest {
                                ["X"] => {
                                    scene_object.object_update.object_data.rotation[0] =
                                        val.parse()?
                                }
                                ["Y"] => {
                                    scene_object.object_update.object_data.rotation[1] =
                                        val.parse()?
                                }
                                ["Z"] => {
                                    scene_object.object_update.object_data.rotation[2] =
                                        val.parse()?
                                }
                                _ => {}
                            },
                            ["Velocity", rest @ ..] => match rest {
                                ["X"] => {
                                    scene_object.object_update.object_data.velocity[0] =
                                        val.parse()?
                                }
                                ["Y"] => {
                                    scene_object.object_update.object_data.velocity[1] =
                                        val.parse()?
                                }
                                ["Z"] => {
                                    scene_object.object_update.object_data.velocity[2] =
                                        val.parse()?
                                }
                                _ => {}
                            },
                            ["AngularVelocity", rest @ ..] => match rest {
                                ["X"] => {
                                    scene_object.object_update.object_data.angular_velocity[0] =
                                        val.parse()?
                                }
                                ["Y"] => {
                                    scene_object.object_update.object_data.angular_velocity[1] =
                                        val.parse()?
                                }
                                ["Z"] => {
                                    scene_object.object_update.object_data.angular_velocity[2] =
                                        val.parse()?
                                }
                                _ => {}
                            },
                            ["Acceleration", rest @ ..] => match rest {
                                ["X"] => {
                                    scene_object.object_update.object_data.acceleration[0] =
                                        val.parse()?
                                }
                                ["Y"] => {
                                    scene_object.object_update.object_data.acceleration[1] =
                                        val.parse()?
                                }
                                ["Z"] => {
                                    scene_object.object_update.object_data.acceleration[2] =
                                        val.parse()?
                                }
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
                                    scene_object.object_update.texture_entry =
                                        val.as_bytes().to_vec()
                                }
                                ["ExtraParams"] => {
                                    scene_object.object_update.extra_params =
                                        val.as_bytes().to_vec()
                                }
                                ["PathBegin"] => {
                                    scene_object.object_update.primitive_geometry.path_begin =
                                        val.parse::<u16>()?
                                }
                                ["PathCurve"] => {
                                    scene_object.object_update.primitive_geometry.path_curve =
                                        val.parse::<u8>()?
                                }
                                ["PathEnd"] => {
                                    scene_object.object_update.primitive_geometry.path_end =
                                        val.parse::<u16>()?
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
                                    scene_object.object_update.primitive_geometry.path_skew =
                                        val.parse::<i8>()?
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
                                    scene_object.object_update.pcode =
                                        ObjectType::from_bytes(&val.parse::<u8>()?);
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
                                    scene_object.object_update.primitive_geometry.profile_shape =
                                        Some(val)
                                }
                                ["HollowShape"] => {
                                    scene_object.object_update.primitive_geometry.hollow_shape =
                                        Some(val)
                                }

                                ["State"] => scene_object.sculpt.state = val.parse::<i32>()?,
                                ["LastAttachPoint"] => {
                                    scene_object.sculpt.last_attach_point = val.parse::<i32>()?
                                }
                                ["SculptTexture", "UUID"] => {
                                    scene_object.sculpt.texture = Uuid::parse_str(&val)?
                                }
                                ["SculptType"] => {
                                    scene_object.sculpt.sculpt_type = val.parse::<i32>()?
                                }
                                ["SculptEntry"] => {
                                    scene_object.sculpt.entry = val.parse::<bool>()?
                                }
                                ["FlexiSoftness"] => {
                                    scene_object.sculpt.flex.softness = val.parse::<i32>()?
                                }
                                ["FlexiTension"] => {
                                    scene_object.sculpt.flex.tension = val.parse::<i32>()?
                                }
                                ["FlexiDrag"] => {
                                    scene_object.sculpt.flex.drag = val.parse::<i32>()?
                                }
                                ["FlexiGravity"] => {
                                    scene_object.sculpt.flex.gravity = val.parse::<i32>()?
                                }
                                ["FlexiWind"] => {
                                    scene_object.sculpt.flex.wind = val.parse::<i32>()?
                                }

                                ["FlexiForceX"] => {
                                    scene_object.sculpt.flex.force[0] = val.parse::<f32>()?
                                }
                                ["FlexiForceY"] => {
                                    scene_object.sculpt.flex.force[1] = val.parse::<f32>()?
                                }
                                ["FlexiForceZ"] => {
                                    scene_object.sculpt.flex.force[2] = val.parse::<f32>()?
                                }
                                ["LightColorR"] => {
                                    scene_object.sculpt.light.color.r = val.parse::<i32>()?
                                }
                                ["LightColorG"] => {
                                    scene_object.sculpt.light.color.g = val.parse::<i32>()?
                                }
                                ["LightColorB"] => {
                                    scene_object.sculpt.light.color.b = val.parse::<i32>()?
                                }
                                ["LightColorA"] => {
                                    scene_object.sculpt.light.color.a = val.parse::<i32>()?
                                }
                                ["LightRadius"] => {
                                    scene_object.sculpt.light.radius = val.parse::<i32>()?
                                }
                                ["LightCutoff"] => {
                                    scene_object.sculpt.light.cutoff = val.parse::<i32>()?
                                }
                                ["LightFalloff"] => {
                                    scene_object.sculpt.light.falloff = val.parse::<i32>()?
                                }
                                ["LightIntensity"] => {
                                    scene_object.sculpt.light.intensity = val.parse::<i32>()?
                                }
                                ["FlexiEntry"] => {
                                    scene_object.sculpt.flex.entry = val.parse::<bool>()?
                                }
                                ["LightEntry"] => {
                                    scene_object.sculpt.light.entry = val.parse::<bool>()?
                                }
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
                                scene_object.item_data.parent_id = if val == "0" {
                                    Uuid::nil()
                                } else {
                                    Uuid::from_str(&val)?
                                };
                            }
                            ["CreationDate"] => {
                                scene_object.item_data.created_at = SystemTime::UNIX_EPOCH
                                    + std::time::Duration::from_secs(val.parse::<u64>()?)
                            }
                            ["Category"] => scene_object.category = val,
                            ["SalePrice"] => {
                                scene_object.item_data.sale_info.price = val.parse::<i32>()?
                            }
                            ["ObjectSaleType"] => {
                                scene_object.item_data.sale_info.sale_type =
                                    SaleType::from_string(&val)
                            }

                            ["OwnershipCost"] => {
                                scene_object.item_data.sale_info.ownership_cost =
                                    Some(val.parse::<i32>()?)
                            }
                            ["GroupID", "UUID"] => {
                                scene_object.item_data.permissions.group_id = Uuid::from_str(&val)?
                            }
                            ["OwnerID", "UUID"] => {
                                scene_object.item_data.permissions.owner_id = Uuid::from_str(&val)?
                            }
                            ["LastOwnerID", "UUID"] => {
                                scene_object.item_data.permissions.last_owner_id =
                                    Some(Uuid::from_str(&val)?)
                            }
                            ["RezzerID", "UUID"] => scene_object.rezzer_id = Uuid::from_str(&val)?,
                            ["BaseMask"] => {
                                scene_object.item_data.permissions.base_mask = val.parse::<i32>()?
                            }
                            ["OwnerMask"] => {
                                scene_object.item_data.permissions.owner_mask =
                                    val.parse::<i32>()?
                            }
                            ["Groupmask"] => {
                                scene_object.item_data.permissions.group_mask =
                                    val.parse::<i32>()?
                            }
                            ["EveryoneMask"] => {
                                scene_object.item_data.permissions.everyone_mask =
                                    val.parse::<i32>()?
                            }
                            ["NextOwnerMask"] => {
                                scene_object.item_data.permissions.next_owner_mask =
                                    val.parse::<i32>()?
                            }
                            ["Flags"] => {
                                if val == "None" {
                                    scene_object.item_data.flags = 0
                                } else {
                                    scene_object.item_data.flags = val.parse::<i32>()?
                                }
                            }
                            ["CollisionSound", "UUID"] => {
                                scene_object.collision_sound = Uuid::from_str(&val)?
                            }
                            ["CollisionSoundVolume"] => {
                                scene_object.collision_sound_volume = val.parse::<i32>()?
                            }
                            ["AttachedPos", rest @ ..] => match rest {
                                ["X"] => scene_object.attached_pos[0] = val.parse()?,
                                ["Y"] => scene_object.attached_pos[1] = val.parse()?,
                                ["Z"] => scene_object.attached_pos[2] = val.parse()?,
                                _ => {}
                            },
                            ["Textureanimation"] => {
                                scene_object.object_update.texture_anim = val.into_bytes()
                            }
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
                            ["VolumeDetectActive"] => {
                                scene_object.volume_detect_active = val.parse::<bool>()?
                            }
                            ["PhysicsShapetype"] => {
                                scene_object.physics_shape_type = val.parse::<i32>()?
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
                            ["SoundGain"] => {
                                scene_object.object_update.sound.gain = val.parse::<f32>()?
                            }
                            ["SoundFlags"] => {
                                scene_object.object_update.sound.flags = val.parse::<u8>()?
                            }
                            ["SoundRadius"] => {
                                scene_object.object_update.sound.radius = val.parse::<f32>()?
                            }
                            ["SoundQueueing"] => {
                                scene_object.sound_queueing = val.parse::<bool>()?
                            }
                            _ => {}
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }

            buf.clear();
        }

        Ok(scene_object)
    }
}
