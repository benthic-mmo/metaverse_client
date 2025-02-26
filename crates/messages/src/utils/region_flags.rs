#[derive(Default)]
pub struct RegionFlags {
    /// Agents can take damage and be killed
    pub allow_damage: bool,
    /// Landmarks can be created here
    pub allow_landmark: bool,
    /// Home position can be set in this sim
    pub allow_set_home: bool,
    /// Home position is reset when an agent teleports away
    pub reset_home_on_teleport: bool,
    /// Sun does not move
    pub sun_fixed: bool,
    /// Allows private parcels (i.e. banlines)
    pub allow_access_override: bool,
    /// Disable heightmap alterations (agents can still plant foliage)
    pub block_terraform: bool,
    /// Land cannot be released, sold, or purchased
    pub block_land_resell: bool,
    /// All content is wiped nightly
    pub sandbox: bool,
    /// Unknown: Related to the availability of an overview world map tile
    pub null_layer: bool,
    /// Unknown: Related to region debug flags
    pub skip_agent_action: bool,
    /// Region does not update agent prim interest lists
    pub skip_update_interest_list: bool,
    /// No collision detection for non-agent objects
    pub skip_collisions: bool,
    /// No scripts are run
    pub skip_scripts: bool,
    /// All physics processing is turned off
    pub skip_physics: bool,
    /// Region can be seen from other regions on world map
    pub externally_visible: bool,
    /// Region can be seen from mainland on world map
    pub mainland_visible: bool,
    /// Agents not explicitly on the access list can visit the region
    pub public_allowed: bool,
    /// Traffic calculations are not run across entire region
    pub block_dwell: bool,
    /// Flight is disabled
    pub no_fly: bool,
    /// Allow direct (p2p) teleporting
    pub allow_direct_teleport: bool,
    /// Estate owner has temporarily disabled scripting
    pub estate_skip_scripts: bool,
    /// Restricts the usage of the LSL llPushObject function
    pub restrict_push_object: bool,
    /// Deny agents with no payment info on file
    pub deny_anonymous: bool,
    /// Deny agents with payment info on file
    pub deny_identified: bool,
    /// Deny agents who have made a monetary transaction
    pub deny_transacted: bool,
    /// Parcels within the region may be joined or divided by anyone
    pub allow_parcel_changes: bool,
    /// Abuse reports sent from within this region are sent to the estate owner defined email
    pub abuse_email_to_estate_owner: bool,
    /// Region is Voice Enabled
    pub allow_voice: bool,
    /// Removes the ability from parcel owners to set their parcels to show in search
    pub block_parcel_search: bool,
    /// Deny agents who have not been age verified from entering the region
    pub deny_age_unverified: bool,
}

impl RegionFlags {
    /// This function initializes the `RegionFlags` struct from a `u64` (representing the raw flags).
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let bits = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        RegionFlags {
            allow_damage: (bits & (1 << 0)) != 0,
            allow_landmark: (bits & (1 << 1)) != 0,
            allow_set_home: (bits & (1 << 2)) != 0,
            reset_home_on_teleport: (bits & (1 << 3)) != 0,
            sun_fixed: (bits & (1 << 4)) != 0,
            allow_access_override: (bits & (1 << 5)) != 0,
            block_terraform: (bits & (1 << 6)) != 0,
            block_land_resell: (bits & (1 << 7)) != 0,
            sandbox: (bits & (1 << 8)) != 0,
            null_layer: (bits & (1 << 9)) != 0,
            skip_agent_action: (bits & (1 << 10)) != 0,
            skip_update_interest_list: (bits & (1 << 11)) != 0,
            skip_collisions: (bits & (1 << 12)) != 0,
            skip_scripts: (bits & (1 << 13)) != 0,
            skip_physics: (bits & (1 << 14)) != 0,
            externally_visible: (bits & (1 << 15)) != 0,
            mainland_visible: (bits & (1 << 16)) != 0,
            public_allowed: (bits & (1 << 17)) != 0,
            block_dwell: (bits & (1 << 18)) != 0,
            no_fly: (bits & (1 << 19)) != 0,
            allow_direct_teleport: (bits & (1 << 20)) != 0,
            estate_skip_scripts: (bits & (1 << 21)) != 0,
            restrict_push_object: (bits & (1 << 22)) != 0,
            deny_anonymous: (bits & (1 << 23)) != 0,
            deny_identified: (bits & (1 << 24)) != 0,
            deny_transacted: (bits & (1 << 25)) != 0,
            allow_parcel_changes: (bits & (1 << 26)) != 0,
            abuse_email_to_estate_owner: (bits & (1 << 27)) != 0,
            allow_voice: (bits & (1 << 28)) != 0,
            block_parcel_search: (bits & (1 << 29)) != 0,
            deny_age_unverified: (bits & (1 << 30)) != 0,
        }
    }
}
