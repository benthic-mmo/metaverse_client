use std::fmt;
use std::path::PathBuf;

static DEFAULT_BASE_HOSTNAME: &str = "127.0.0.1";
static DEFAULT_HOST: &str = "localhost";

// this is the full definition for a simulator INI, taken from OpenSimulator's INI files.
#[derive(Default)]
pub struct SimulatorConfig {
    pub config_const: ConfigConst,
    pub startup: Startup,
    pub access_control: AccessControl,
    pub map: Map,
    pub permissions: Permissions,
    pub estates: Estates,
    pub smtp: SMTP,
    pub network: Network,
    pub xmlrpc: XMLRPC,
    pub client_stack_linden_udp: ClientStackLindenUDP,
    pub client_stack_linden_caps: ClientStackLindenCaps,
    pub simulator_features: SimulatorFeatures,
    pub chat: Chat,
    pub entity_transfer: EntityTransfer,
    pub messaging: Messaging,
    pub bullet_sim: BulletSim,
    pub ode_physics_settings: ODEPhysicsSettings,
    pub remote_admin: RemoteAdmin,
    pub wind: Wind,
    pub materials: Materials,
    pub data_snapshot: DataSnapshot,
    pub economy: Economy,
    pub y_engine: YEngine,
    pub x_engine: XEngine,
    pub ossl: OSSL,
    pub free_switch_voice: FreeSwitchVoice,
    pub groups: Groups,
    pub interest_management: InterestManagement,
    pub media_on_a_prim: MediaOnAPrim,
    pub npc: NPC,
    pub terrain: Terrain,
    pub land_management: LandManagement,
    pub user_profiles: UserProfiles,
    pub x_bakes: XBakes,
    pub god_names: GodNames,
    pub architecture: Architecture,
}
impl SimulatorConfig {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                result.push_str(&format!("[{}]\n{}\n", $key, $value));
            };
        }

        append_line!("Const", self.config_const.to_string());
        append_line!("Startup", self.startup.to_string());
        append_line!("AccessControl", self.access_control.to_string());
        append_line!("Map", self.map.to_string());
        append_line!("Permissions", self.permissions.to_string());
        append_line!("Estates", self.estates.to_string());
        append_line!("SMTP", self.smtp.to_string());
        append_line!("Network", self.network.to_string());
        append_line!("XMLRPC", self.xmlrpc.to_string());
        append_line!(
            "ClientStack.LindenUDP",
            self.client_stack_linden_udp.to_string()
        );
        append_line!(
            "ClientStack.LindenCaps",
            self.client_stack_linden_caps.to_string()
        );
        append_line!("SimulatorFeatures", self.simulator_features.to_string());
        append_line!("Chat", self.chat.to_string());
        append_line!("EntityTransfer", self.entity_transfer.to_string());
        append_line!("Messaging", self.messaging.to_string());
        append_line!("BulletSim", self.bullet_sim.to_string());
        append_line!("ODEPhysicsSettings", self.ode_physics_settings.to_string());
        append_line!("RemoteAdmin", self.remote_admin.to_string());
        append_line!("Wind", self.wind.to_string());
        append_line!("Materials", self.materials.to_string());
        append_line!("DataSnapshot", self.data_snapshot.to_string());
        append_line!("Economy", self.economy.to_string());
        append_line!("YEngine", self.y_engine.to_string());
        append_line!("XEngine", self.x_engine.to_string());
        append_line!("OSSL", self.ossl.to_string());
        append_line!("FreeSwitchVoice", self.free_switch_voice.to_string());
        append_line!("Groups", self.groups.to_string());
        append_line!("InterestManagement", self.interest_management.to_string());
        append_line!("MediaOnAPrim", self.media_on_a_prim.to_string());
        append_line!("NPC", self.npc.to_string());
        append_line!("Terrain", self.terrain.to_string());
        append_line!("LandManagement", self.land_management.to_string());
        append_line!("UserProfiles", self.user_profiles.to_string());
        append_line!("XBakes", self.x_bakes.to_string());
        append_line!("GodNames", self.god_names.to_string());
        append_line!("Architecture", self.architecture.to_string());

        result
    }
}

pub enum Mesher {
    Meshmerizer,
    ZeroMesher,
    UbODEMeshmerizer,
}
impl Mesher {
    pub fn to_string(&self) -> String {
        match self {
            Mesher::Meshmerizer => "Meshmerizer".to_string(),
            Mesher::ZeroMesher => "ZeroMesher".to_string(),
            Mesher::UbODEMeshmerizer => "ubdOEMeshmerizer".to_string(),
        }
    }
}

pub enum PhysicsEngine {
    BulletSim,
    OpenDynamicsEngine,
    BasicPhysics,
    POS,
    UbODE,
}
impl PhysicsEngine {
    pub fn to_string(&self) -> String {
        match self {
            PhysicsEngine::BulletSim => "BulletSim".to_string(),
            PhysicsEngine::OpenDynamicsEngine => "OpenDynamicsEngine".to_string(),
            PhysicsEngine::BasicPhysics => "basicphysics".to_string(),
            PhysicsEngine::POS => "POS".to_string(),
            PhysicsEngine::UbODE => "ubODE".to_string(),
        }
    }
}

pub enum ScriptEngine {
    XEngine,
    YEngine,
}
impl ScriptEngine {
    pub fn to_string(&self) -> String {
        match self {
            ScriptEngine::XEngine => "XEngine".to_string(),
            ScriptEngine::YEngine => "YEngine".to_string(),
        }
    }
}

pub enum SpawnPointRouting {
    Closest,
    Random,
    Sequence,
}
impl SpawnPointRouting {
    pub fn to_string(&self) -> String {
        match self {
            SpawnPointRouting::Closest => "closest".to_string(),
            SpawnPointRouting::Random => "random".to_string(),
            SpawnPointRouting::Sequence => "sequence".to_string(),
        }
    }
}

pub enum MapImageModule {
    MapImageModule,
    Warp3DImageModule,
}
impl MapImageModule {
    pub fn to_string(&self) -> String {
        match self {
            MapImageModule::MapImageModule => "MapImageModule".to_string(),
            MapImageModule::Warp3DImageModule => "Warp3DImageModule".to_string(),
        }
    }
}

pub enum PermissionsModule {
    DefaultPermissionsModule,
    PrimLimitsModule,
}
impl PermissionsModule {
    pub fn to_string(&self) -> String {
        match self {
            PermissionsModule::DefaultPermissionsModule => "DefaultPermissionsModule".to_string(),
            PermissionsModule::PrimLimitsModule => "PrimLimitsModule".to_string(),
        }
    }
}

pub enum XMLRPCRouterModule {
    XMLRpcRouterModule,
    XMLRpcGridRouterModule,
}
impl XMLRPCRouterModule {
    pub fn to_string(&self) -> String {
        match self {
            XMLRPCRouterModule::XMLRpcRouterModule => "XmlRpcRouterModule".to_string(),
            XMLRPCRouterModule::XMLRpcGridRouterModule => "XmlRpcGridRouterModule".to_string(),
        }
    }
}

pub enum LandingPointBehavior {
    OpenSimulator,
    SecondLife,
}
impl LandingPointBehavior {
    pub fn to_string(&self) -> String {
        match self {
            LandingPointBehavior::OpenSimulator => "LandingPointBehavior_OS".to_string(),
            LandingPointBehavior::SecondLife => "LandingPointBehavior_SL".to_string(),
        }
    }
}

pub enum OfflineMessageModule {
    OfflineMessageModule,
    OfflineMessageModuleV2,
}
impl OfflineMessageModule {
    pub fn to_string(&self) -> String {
        match self {
            OfflineMessageModule::OfflineMessageModule => "OfflineMessageModule".to_string(),
            OfflineMessageModule::OfflineMessageModuleV2 => "Offline Message Module V2".to_string(),
        }
    }
}

pub enum DataExposure {
    Minimum,
    All,
}
impl DataExposure {
    pub fn to_string(&self) -> String {
        match self {
            DataExposure::Minimum => "minimum".to_string(),
            DataExposure::All => "all".to_string(),
        }
    }
}

pub enum RegionInfoSource {
    Web,
    Filesystem,
}
impl RegionInfoSource {
    pub fn to_string(&self) -> String {
        match self {
            RegionInfoSource::Web => "web".to_string(),
            RegionInfoSource::Filesystem => "filesystem".to_string(),
        }
    }
}

pub enum XEnginePriority {
    Lowest,
    BelowNormal,
    Normal,
    AboveNormal,
    Highest,
}
impl XEnginePriority {
    pub fn to_string(&self) -> String {
        match self {
            XEnginePriority::Lowest => "Lowest".to_string(),
            XEnginePriority::BelowNormal => "BelowNormal".to_string(),
            XEnginePriority::Normal => "Normal".to_string(),
            XEnginePriority::AboveNormal => "AboveNormal".to_string(),
            XEnginePriority::Highest => "Highest".to_string(),
        }
    }
}

pub enum ScriptStopStrategy {
    Abort,
    CoOp,
}
impl ScriptStopStrategy {
    pub fn to_string(&self) -> String {
        match self {
            ScriptStopStrategy::Abort => "abort".to_string(),
            ScriptStopStrategy::CoOp => "co-op".to_string(),
        }
    }
}

pub enum GroupsModule {
    Default,
    GroupsModuleV2,
}
impl GroupsModule {
    pub fn to_string(&self) -> String {
        match self {
            GroupsModule::Default => "Default".to_string(),
            GroupsModule::GroupsModuleV2 => "Groups Module V2".to_string(),
        }
    }
}

pub enum ServicesConnectorModule {
    XMLRPCGroupsServicesConnector,
    GroupsLocalServiceConnector,
    GroupsRemoteServiceConnector,
    GroupsHGServiceConnector,
}
impl ServicesConnectorModule {
    pub fn to_string(&self) -> String {
        match self {
            ServicesConnectorModule::XMLRPCGroupsServicesConnector => {
                "XmlRpcGroupsServicesConnector".to_string()
            }
            ServicesConnectorModule::GroupsLocalServiceConnector => {
                "Groups Local Service Connector".to_string()
            }
            ServicesConnectorModule::GroupsRemoteServiceConnector => {
                "Groups Remote Service Connector".to_string()
            }
            ServicesConnectorModule::GroupsHGServiceConnector => {
                "Groups HG Service Connector".to_string()
            }
        }
    }
}

pub enum LocalService {
    Local,
    Remote,
}
impl LocalService {
    pub fn to_string(&self) -> String {
        match self {
            LocalService::Local => "local".to_string(),
            LocalService::Remote => "remote".to_string(),
        }
    }
}

pub enum GroupsMessagingModule {
    GroupsMessagingModule,
    GroupsMessagingModuleV2,
}
impl GroupsMessagingModule {
    pub fn to_string(&self) -> String {
        match self {
            GroupsMessagingModule::GroupsMessagingModule => "GroupsMessagingModule".to_string(),
            GroupsMessagingModule::GroupsMessagingModuleV2 => {
                "Groups Messaging Module V2".to_string()
            }
        }
    }
}

pub enum UpdatePrioritizationScheme {
    BestAvatarResponsiveness,
    SimpleAngularDistance,
}
impl UpdatePrioritizationScheme {
    pub fn to_string(&self) -> String {
        match self {
            UpdatePrioritizationScheme::BestAvatarResponsiveness => {
                "BestAvatarResponsiveness".to_string()
            }
            UpdatePrioritizationScheme::SimpleAngularDistance => {
                "SimpleAngularDistance".to_string()
            }
        }
    }
}

pub enum Architectures {
    Standalone,
    StandaloneHypergrid,
    Grid,
    GridHypergrid,
}
impl Architectures {
    pub fn as_pathbuf(&self) -> PathBuf {
        match self {
            Architectures::Standalone => PathBuf::from("config-include/Standalone.ini"),
            Architectures::StandaloneHypergrid => {
                PathBuf::from("config-include/StandaloneHypergrid.ini")
            }
            Architectures::Grid => PathBuf::from("config-include/Grid.ini"),
            Architectures::GridHypergrid => PathBuf::from("config-include/GridHypergrid.ini"),
        }
    }
    fn to_string(&self) -> String {
        self.as_pathbuf().to_string_lossy().into_owned()
    }
}

pub struct PermissionsModuleList(Vec<PermissionsModule>);
impl fmt::Display for PermissionsModuleList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted_modules: Vec<String> = self
            .0
            .iter()
            .map(|module| match module {
                PermissionsModule::DefaultPermissionsModule => {
                    "DefaultPermissionsModule".to_string()
                }
                PermissionsModule::PrimLimitsModule => "PrimLimitsModule".to_string(),
            })
            .collect();

        write!(f, "{}", formatted_modules.join(", "))
    }
}

pub struct HttpProxyExceptionsList(pub Vec<String>);
impl fmt::Display for HttpProxyExceptionsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(";"))
    }
}

pub struct AllowedClientsList(Vec<String>);
impl fmt::Display for AllowedClientsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct DeniedClientsList(Vec<String>);
impl fmt::Display for DeniedClientsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct OutboundDisallowForUserScriptsExceptList(Vec<String>);
impl fmt::Display for OutboundDisallowForUserScriptsExceptList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct AccessIpAddressesList(Vec<String>);
impl fmt::Display for AccessIpAddressesList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct EnabledMethodsList(Vec<String>);
impl fmt::Display for EnabledMethodsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct DataServicesList(Vec<String>);
impl fmt::Display for DataServicesList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(";"))
    }
}

pub struct DataSrvMiSearchList(Vec<String>);
impl fmt::Display for DataSrvMiSearchList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(";"))
    }
}

pub struct GodFullNamesList(Vec<String>);
impl fmt::Display for GodFullNamesList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct GodSurnamesList(Vec<String>);
impl fmt::Display for GodSurnamesList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct ConfigConst {
    pub base_host_name: String,
    pub base_url: String,
    pub public_port: i32,
    pub priv_url: String,
    pub private_port: i32,
}
impl ConfigConst {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                result.push_str(&format!("{} = \"{}\"\n", $key, $value.to_string()));
            };
        }

        append_line!("BaseHostName", &self.base_host_name);
        append_line!("BaseURL", &self.base_url);
        append_line!("PublicPort", &self.public_port);
        append_line!("PrivURL", &self.priv_url);
        append_line!("PrivatePort", &self.private_port);

        result
    }
}
impl Default for ConfigConst {
    fn default() -> Self {
        ConfigConst {
            base_host_name: DEFAULT_BASE_HOSTNAME.to_string(),
            base_url: format!("http://{}", DEFAULT_BASE_HOSTNAME).to_string(),
            public_port: 9000,
            priv_url: DEFAULT_BASE_HOSTNAME.to_string(),
            private_port: 8003,
        }
    }
}

#[derive(Default)]
pub struct Startup {
    pub console_prompt: Option<String>,
    pub console_history_file_enabled: Option<bool>,
    pub console_history_file: Option<PathBuf>,
    pub console_history_file_lines: Option<i32>,
    pub console_history_time_stamp: Option<bool>,
    pub save_crashes: Option<bool>,
    pub crash_dir: Option<PathBuf>,
    pub pid_file: Option<PathBuf>,
    pub registry_location: Option<PathBuf>,
    pub config_directory: Option<PathBuf>,
    pub region_info_source: Option<RegionInfoSource>,
    pub region_load_regions_dir: Option<PathBuf>,
    pub region_load_webserver_url: Option<String>,
    pub allow_regionless: Option<bool>,
    pub non_physical_prim_min: Option<f32>,
    pub non_physical_prim_max: Option<f32>,
    pub physical_prim_min: Option<f32>,
    pub physical_prim_max: Option<f32>,
    pub clamp_prim_size: Option<bool>,
    pub link_set_prims: Option<f32>,
    pub allow_script_crossing: Option<bool>,
    pub trust_binaries: Option<bool>,
    pub in_world_restart_shuts_down: Option<bool>,
    pub minimum_time_before_persistence_considered: Option<i32>,
    pub maximum_time_before_persistence_considered: Option<i32>,
    pub physical_prim: Option<bool>,
    pub meshing: Option<Mesher>,
    pub physics: Option<PhysicsEngine>,
    pub default_script_engine: Option<ScriptEngine>,
    pub http_proxy: Option<String>,
    pub http_proxy_exceptions: Option<HttpProxyExceptionsList>,
    pub email_module: Option<String>,
    pub spawn_point_routing: Option<SpawnPointRouting>,
    pub tele_hub_allow_landmark: Option<bool>,
    pub no_verify_cert_chain: Option<bool>,
    pub no_verify_cert_host_name: Option<bool>,
}
impl Startup {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("ConsolePrompt", &self.console_prompt);
        append_line!(
            "ConsoleHistoryFileEnabled",
            &self.console_history_file_enabled
        );
        append_line!(
            "ConsoleHistoryFile",
            &self
                .console_history_file
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!("ConsoleHistoryFileLines", &self.console_history_file_lines);
        append_line!("ConsoleHistoryTimeStamp", &self.console_history_time_stamp);
        append_line!("save_crashes", &self.save_crashes);
        append_line!(
            "crash_dir",
            self.crash_dir.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "PIDFile",
            self.pid_file.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "RegistryLocation",
            self.registry_location.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "ConfigDirectory",
            self.config_directory.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!("region_info_source", &self.region_info_source);
        append_line!(
            "regionload_regionsdir",
            self.region_load_regions_dir
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!("regionload_webserver_url", &self.region_load_webserver_url);
        append_line!("allow_regionless", &self.allow_regionless);
        append_line!("NonPhysicalPrimMin", &self.non_physical_prim_min);
        append_line!("NonPhysicalPrimMax", &self.non_physical_prim_max);
        append_line!("PhysicalPrimMin", &self.physical_prim_min);
        append_line!("PhysicalPrimMax", &self.physical_prim_max);
        append_line!("ClampPrimSize", &self.clamp_prim_size);
        append_line!("LinksetPrims", &self.link_set_prims);
        append_line!("AllowScriptCrossing", &self.allow_script_crossing);
        append_line!("TrustBinaries", &self.trust_binaries);
        append_line!("InWorldRestartShutsDown", &self.in_world_restart_shuts_down);
        append_line!(
            "MinimumTimeBeforePersistenceConsidered",
            &self.minimum_time_before_persistence_considered
        );
        append_line!(
            "MaximumTimeBeforePersistenceConsidered",
            &self.maximum_time_before_persistence_considered
        );
        append_line!("physical_prim", &self.physical_prim);
        append_line!("meshing", &self.meshing);
        append_line!("physics", &self.physics);
        append_line!("DefaultScriptEngine", &self.default_script_engine);
        append_line!("HttpProxy", &self.http_proxy);
        append_line!("HttpProxyExceptions", &self.http_proxy_exceptions);
        append_line!("emailmodule", &self.email_module);
        append_line!("SpawnPointRouting", &self.spawn_point_routing);
        append_line!("TeleHubAllowLandmark", &self.tele_hub_allow_landmark);
        append_line!("NoVerifyCertChain", &self.no_verify_cert_chain);
        append_line!("NoVerifyCertHostname", &self.no_verify_cert_host_name);
        result
    }
}

#[derive(Default)]
pub struct AccessControl {
    pub allowed_clients: Option<AllowedClientsList>,
    pub denied_clients: Option<DeniedClientsList>,
}
impl AccessControl {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("AllowedClients", &self.allowed_clients);
        append_line!("DeniedClients", &self.denied_clients);

        result
    }
}

#[derive(Default)]
pub struct Map {
    pub generate_map_tiles: Option<bool>,
    pub map_image_module: Option<MapImageModule>,
    pub map_tile_refresh: Option<i32>,
    pub map_tile_static_uuid: Option<String>,
    pub texture_on_map_tile: Option<bool>,
    pub draw_prim_on_map_tile: Option<bool>,
    pub texture_prims: Option<bool>,
    pub texture_prim_size: Option<i32>,
    pub render_meshes: Option<bool>,
    pub map_color_water: Option<String>,
    pub map_color_1: Option<String>,
    pub map_color_2: Option<String>,
    pub map_color_3: Option<String>,
    pub map_color_4: Option<String>,
}
impl Map {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("GenerateMaptiles", &self.generate_map_tiles);
        append_line!("MapImageModule", &self.map_image_module);
        append_line!("MaptileRefresh", &self.map_tile_refresh);
        append_line!("MaptileStaticUUID", &self.map_tile_static_uuid);
        append_line!("TextureOnMapTile", &self.texture_on_map_tile);
        append_line!("DrawPrimOnMapTile", &self.draw_prim_on_map_tile);
        append_line!("TexturePrims", &self.texture_prims);
        append_line!("TexturePrimSize", &self.texture_prim_size);
        append_line!("RenderMeshes", &self.render_meshes);
        append_line!("MapColorWater", &self.map_color_water);
        append_line!("MapColor1", &self.map_color_1);
        append_line!("MapColor2", &self.map_color_2);
        append_line!("MapColor3", &self.map_color_3);
        append_line!("MapColor4", &self.map_color_4);

        result
    }
}

pub struct Permissions {
    pub permission_modules: Option<PermissionsModuleList>,
    pub server_side_object_permissions: Option<bool>,
    pub automatic_gods: Option<bool>,
    pub implicit_gods: Option<bool>,
    pub allow_grid_gods: Option<bool>,
    pub region_owner_is_god: Option<bool>,
    pub region_manager_is_god: Option<bool>,
    pub simple_build_permissions: Option<bool>,
}
impl Permissions {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("permissionmodules", &self.permission_modules);
        append_line!(
            "server_side_object_permissions",
            &self.server_side_object_permissions
        );
        append_line!("automatic_gods", &self.automatic_gods);
        append_line!("implicit_gods", &self.implicit_gods);
        append_line!("allow_grid_gods", &self.allow_grid_gods);
        append_line!("region_owner_is_god", &self.region_owner_is_god);
        append_line!("region_manager_is_god", &self.region_manager_is_god);
        append_line!("simple_build_permissions", &self.simple_build_permissions);

        result
    }
}
impl Default for Permissions {
    fn default() -> Self {
        Permissions {
            automatic_gods: Some(false),
            implicit_gods: Some(false),
            allow_grid_gods: Some(true),
            permission_modules: Default::default(),
            server_side_object_permissions: Default::default(),
            region_owner_is_god: Default::default(),
            region_manager_is_god: Default::default(),
            simple_build_permissions: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Estates {
    pub default_estate_name: Option<String>,
    pub default_estate_owner_name: Option<String>,
    pub default_estate_owner_uuid: Option<String>,
    pub default_estate_owner_email: Option<String>,
    pub default_estate_owner_password: Option<String>,
}
impl Estates {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("DefaultEstateName", &self.default_estate_name);
        append_line!("DefaultEstateOwnerName", &self.default_estate_owner_name);
        append_line!("DefaultEstateOwnerUUID", &self.default_estate_owner_uuid);
        append_line!("DefaultEstateOwnerEmail", &self.default_estate_owner_email);
        append_line!(
            "DefaultEstateOwnerPassword",
            &self.default_estate_owner_password
        );

        result
    }
}

#[derive(Default)]
pub struct SMTP {
    pub enabled: Option<bool>,
    pub enable_email_to_external_objects: Option<bool>,
    pub mails_from_owner_per_hour: Option<i32>,
    pub mails_to_prim_address_per_hour: Option<i32>,
    pub smtp_mails_per_day: Option<i32>,
    pub mails_to_smtp_address_per_hour: Option<i32>,
    pub enable_email_to_smtp: Option<bool>,
    pub internal_object_host: Option<String>,
    pub host_domain_header_from: Option<String>,
    pub smtp_server_from: Option<String>,
    pub email_pause_time: Option<i32>,
    pub email_max_size: Option<i32>,
    pub smtp_server_tls: Option<bool>,
    pub smtp_server_hostname: Option<String>,
    pub smtp_server_port: Option<i32>,
    pub smtp_server_login: Option<String>,
    pub smtp_server_password: Option<String>,
    pub smtp_verify_cert_chain: Option<bool>,
    pub smtp_verify_cert_names: Option<bool>,
}
impl SMTP {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value));
                }
            };
        }

        append_line!("enabled", &self.enabled);
        append_line!(
            "enableEmailToExternalObjects",
            &self.enable_email_to_external_objects
        );
        append_line!("MailsFromOwnerPerHour", &self.mails_from_owner_per_hour);
        append_line!(
            "MailsToPrimAddressPerHour",
            &self.mails_to_prim_address_per_hour
        );
        append_line!("SMTP_MailsPerDay", &self.smtp_mails_per_day);
        append_line!(
            "MailsToSMTPAddressPerHour",
            &self.mails_to_smtp_address_per_hour
        );
        append_line!("enableEmailToSMTP", &self.enable_email_to_smtp);
        append_line!("internal_object_host", &self.internal_object_host);
        append_line!("host_domain_header_from", &self.host_domain_header_from);
        append_line!("SMTP_SERVER_FROM", &self.smtp_server_from);
        append_line!("email_pause_time", &self.email_pause_time);
        append_line!("email_max_size", &self.email_max_size);
        append_line!("SMTP_SERVER_TLS", &self.smtp_server_tls);
        append_line!("SMTP_SERVER_HOSTNAME", &self.smtp_server_hostname);
        append_line!("SMTP_SERVER_PORT", &self.smtp_server_port);
        append_line!("SMTP_SERVER_LOGIN", &self.smtp_server_login);
        append_line!("SMTP_SERVER_PASSWORD", &self.smtp_server_password);
        append_line!("SMTP_VerifyCertChain", &self.smtp_verify_cert_chain);
        append_line!("SMTP_VerifyCertNames", &self.smtp_verify_cert_names);

        result
    }
}

pub struct Network {
    pub console_user: Option<String>,
    pub console_pass: Option<String>,
    pub console_port: Option<i32>,
    pub http_listener_port: Option<i32>,
    pub http_listener_ssl: Option<bool>,
    pub http_listener_ssl_port: Option<i32>,
    pub http_listener_cn: Option<String>,
    pub http_listener_cert_path: Option<PathBuf>,
    pub http_listener_cert_pass: Option<String>,
    pub outbound_disallow_for_user_scripts_except: Option<OutboundDisallowForUserScriptsExceptList>,
    pub http_body_max_len_max: Option<i32>,
    pub external_host_name_for_lsl: Option<String>,
    pub shard: Option<String>,
    pub user_agent: Option<String>,
    pub auth_type: Option<String>,
    pub http_auth_username: Option<String>,
    pub http_auth_password: Option<String>,
}
impl Network {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("ConsoleUser", &self.console_user);
        append_line!("ConsolePass", &self.console_pass);
        append_line!("console_port", &self.console_port);
        append_line!("http_listener_port", &self.http_listener_port);
        append_line!("http_listener_ssl", &self.http_listener_ssl);
        append_line!("http_listener_sslport", &self.http_listener_ssl_port);
        append_line!("http_listener_cn", &self.http_listener_cn);
        append_line!(
            "http_listener_cert_path",
            &self
                .http_listener_cert_path
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!("http_listener_cert_pass", &self.http_listener_cert_pass);
        append_line!(
            "OutboundDisallowForUserScriptsExcept",
            &self.outbound_disallow_for_user_scripts_except
        );
        append_line!("HttpBodyMaxLenMAX", &self.http_body_max_len_max);
        append_line!("ExternalHostNameForLSL", &self.external_host_name_for_lsl);
        append_line!("shard", &self.shard);
        append_line!("user_agent", &self.user_agent);
        append_line!("AuthType", &self.auth_type);
        append_line!("HttpAuthUsername", &self.http_auth_username);
        append_line!("HttpAuthPassword", &self.http_auth_password);

        result
    }
}
impl Default for Network {
    fn default() -> Self {
        Network {
            external_host_name_for_lsl: Some(DEFAULT_BASE_HOSTNAME.to_string()),
            shard: Some("OpenSim".to_string()),
            console_user: Default::default(),
            console_pass: Default::default(),
            console_port: Default::default(),
            http_listener_port: Default::default(),
            http_listener_ssl: Default::default(),
            http_listener_ssl_port: Default::default(),
            http_listener_cn: Default::default(),
            http_listener_cert_path: Default::default(),
            http_listener_cert_pass: Default::default(),
            outbound_disallow_for_user_scripts_except: Default::default(),
            http_body_max_len_max: Default::default(),
            user_agent: Default::default(),
            auth_type: Default::default(),
            http_auth_username: Default::default(),
            http_auth_password: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct XMLRPC {
    pub xml_rpc_router_module: Option<XMLRPCRouterModule>,
    pub xml_rpc_port: Option<i32>,
    pub xml_rpc_hub_uri: Option<String>,
}
impl XMLRPC {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("XmlRpcRouterModule", &self.xml_rpc_router_module);
        append_line!("XmlRpcPort", &self.xml_rpc_port);
        append_line!("XmlRpcHubURI", &self.xml_rpc_hub_uri);

        result
    }
}

#[derive(Default)]
pub struct ClientStackLindenUDP {
    pub disable_face_lights: Option<bool>,
}
impl ClientStackLindenUDP {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("disableFaceLights", &self.disable_face_lights);

        result
    }
}

pub struct ClientStackLindenCaps {
    pub cap_get_texture: Option<String>,
    pub cap_get_mesh: Option<String>,
    pub cap_avatar_picker_search: Option<String>,
    pub cap_get_display_names: Option<String>,
}
impl ClientStackLindenCaps {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Cap_GetTexture", &self.cap_get_texture);
        append_line!("Cap_GetMesh", &self.cap_get_mesh);
        append_line!("Cap_GetDisplayNames", &self.cap_get_display_names);
        result
    }
}
impl Default for ClientStackLindenCaps {
    fn default() -> Self {
        ClientStackLindenCaps {
            cap_get_texture: Some(DEFAULT_HOST.to_string()),
            cap_get_mesh: Some(DEFAULT_HOST.to_string()),
            cap_avatar_picker_search: Some(DEFAULT_HOST.to_string()),
            cap_get_display_names: Some(DEFAULT_HOST.to_string()),
        }
    }
}

#[derive(Default)]
pub struct SimulatorFeatures {
    pub search_server_uri: Option<String>,
    pub destination_guide_uri: Option<String>,
}
impl SimulatorFeatures {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("SearchServerURI", &self.search_server_uri);
        append_line!("DestinationGuideURI", &self.destination_guide_uri);

        result
    }
}

#[derive(Default)]
pub struct Chat {
    pub whisper_distance: Option<i32>,
    pub say_distance: Option<i32>,
    pub shout_distance: Option<i32>,
}
impl Chat {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("whisper_distance", &self.whisper_distance);
        append_line!("say_distance", &self.say_distance);
        append_line!("shout_distance", &self.shout_distance);

        result
    }
}

#[derive(Default)]
pub struct EntityTransfer {
    pub disable_inter_region_teleport_cancellation: Option<bool>,
    pub landing_point_behavior: Option<LandingPointBehavior>,
}
impl EntityTransfer {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!(
            "DisableInterRegionTeleportCancellation",
            &self.disable_inter_region_teleport_cancellation
        );
        append_line!("LandingPointBehavior", &self.landing_point_behavior);

        result
    }
}

#[derive(Default)]
pub struct Messaging {
    pub offline_message_module: Option<OfflineMessageModule>,
    pub offline_message_url: Option<String>,
    pub storage_provider: Option<String>, // TODO: This may eventually need to be an enum
    pub mute_list_module: Option<String>,
    pub forward_offline_group_messages: Option<bool>,
}
impl Messaging {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("OfflineMessageModule", &self.offline_message_module);
        append_line!("OfflineMessageURL", &self.offline_message_url);
        append_line!("StorageProvider", &self.storage_provider);
        append_line!("MuteListModule", &self.mute_list_module);
        append_line!(
            "ForwardOfflineGroupmessages",
            &self.forward_offline_group_messages
        );

        result
    }
}

pub struct BulletSim {
    pub avatar_to_avatar_collisions_by_default: Option<bool>,
}
impl BulletSim {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!(
            "avatarToAvatarCollisionsByDefault",
            &self.avatar_to_avatar_collisions_by_default
        );

        result
    }
}
impl Default for BulletSim {
    fn default() -> Self {
        BulletSim {
            avatar_to_avatar_collisions_by_default: Some(true),
        }
    }
}

#[derive(Default)]
pub struct ODEPhysicsSettings {
    pub mesh_sculpted_prim: Option<bool>,
}
impl ODEPhysicsSettings {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("mesh_sculpted_prim", &self.mesh_sculpted_prim);

        result
    }
}

#[derive(Default)]
pub struct RemoteAdmin {
    pub enabled: Option<bool>,
    pub port: Option<i32>,
    pub access_password: Option<String>,
    pub access_ip_addresses: Option<AccessIpAddressesList>,
    pub create_region_enable_voice: Option<bool>,
    pub create_region_public: Option<bool>,
    pub enabled_methods: Option<EnabledMethodsList>, // TODO: Create an enum of methods
    pub default_male: Option<PathBuf>,
    pub default_female: Option<PathBuf>,
    pub copy_folders: Option<bool>,
    pub default_appearance: Option<PathBuf>,
}
impl RemoteAdmin {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("enabled", &self.enabled);
        append_line!("port", &self.port);
        append_line!("access_password", &self.access_password);
        append_line!("access_ip_addresses", &self.access_ip_addresses);
        append_line!(
            "create_region_enable_voice",
            &self.create_region_enable_voice
        );
        append_line!("create_region_public", &self.create_region_public);
        append_line!("enabled_methods", &self.enabled_methods);
        append_line!(
            "default_male",
            &self.default_male.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "default_female",
            &self.default_female.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!("copy_folders", &self.copy_folders);
        append_line!(
            "default_appearance",
            &self
                .default_appearance
                .as_ref()
                .map(|p| p.to_string_lossy())
        );

        result
    }
}

#[derive(Default)]
pub struct Wind {
    pub enabled: Option<bool>,
    pub wind_update_rate: Option<i32>,
    pub wind_plugin: Option<String>,
    pub avg_strength: Option<i32>,
    pub avg_direction: Option<i32>,
    pub var_strength: Option<i32>,
    pub var_direction: Option<i32>,
    pub rate_change: Option<i32>,
    pub strength: Option<i32>,
}
impl Wind {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("enabled", &self.enabled);
        append_line!("wind_update_rate", &self.wind_update_rate);
        append_line!("wind_plugin", &self.wind_plugin);
        append_line!("avg_strength", &self.avg_strength);
        append_line!("avg_direction", &self.avg_direction);
        append_line!("var_strength", &self.var_strength);
        append_line!("var_direction", &self.var_direction);
        append_line!("rate_change", &self.rate_change);
        append_line!("strength", &self.strength);

        result
    }
}

#[derive(Default)]
pub struct Materials {
    pub enabled: Option<bool>,
    pub max_materials_per_transaction: Option<i32>,
}
impl Materials {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value));
                }
            };
        }

        append_line!("enable_materials", &self.enabled);
        append_line!(
            "MaxMaterialsPerTransaction",
            &self.max_materials_per_transaction
        );

        result
    }
}

#[derive(Default)]
pub struct DataSnapshot {
    pub index_sims: Option<bool>,
    pub data_exposure: Option<DataExposure>,
    pub gridname: Option<String>,
    pub default_snapshot_period: Option<i32>,
    pub snapshot_cache_directory: Option<PathBuf>,
    pub data_services: Option<DataServicesList>,
    pub data_srv_mi_search: Option<DataSrvMiSearchList>,
}
impl DataSnapshot {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("index_sims", &self.index_sims);
        append_line!("data_exposure", &self.data_exposure);
        append_line!("gridname", &self.gridname);
        append_line!("default_snapshot_period", &self.default_snapshot_period);
        append_line!(
            "snapshot_cache_directory",
            &self
                .snapshot_cache_directory
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!("data_services", &self.data_services);
        append_line!("data_srv_mi_search", &self.data_srv_mi_search);

        result
    }
}

#[derive(Default)]
pub struct Economy {
    pub economy_module: Option<String>,
    pub economy: Option<String>,
    pub sell_enabled: Option<bool>,
    pub price_upload: Option<i32>,
    pub price_group_create: Option<i32>,
}

impl Economy {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("economymodule", &self.economy_module);
        append_line!("economy", &self.economy);
        append_line!("SellEnabled", &self.sell_enabled);
        append_line!("PriceUpload", &self.price_upload);
        append_line!("PriceGroupCreate", &self.price_group_create);

        result
    }
}

#[derive(Default)]
pub struct YEngine {
    pub enabled: Option<bool>,
    pub min_timer_interval: Option<i32>,
    pub script_delay_factor: Option<i32>,
    pub script_distance_limit_factor: Option<i32>,
    pub notecard_line_read_chars_max: Option<i32>,
    pub sensor_max_range: Option<i32>,
    pub sensor_max_results: Option<i32>,
    pub script_engines_path: Option<PathBuf>,
}
impl YEngine {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("MinTimerInterval", &self.min_timer_interval);
        append_line!("ScriptDelayFactor", &self.script_delay_factor);
        append_line!(
            "ScriptDistanceLimitFactor",
            &self.script_distance_limit_factor
        );
        append_line!(
            "NotecardLineReadCharsMax",
            &self.notecard_line_read_chars_max
        );
        append_line!("SensorMaxRange", &self.sensor_max_range);
        append_line!("SensorMaxResults", &self.sensor_max_results);
        append_line!(
            "ScriptEnginesPath",
            &self
                .script_engines_path
                .as_ref()
                .map(|p| p.to_string_lossy())
        );

        result
    }
}

pub struct XEngine {
    pub enabled: Option<bool>,
    pub min_threads: Option<i32>,
    pub max_threads: Option<i32>,
    pub idle_timeout: Option<i32>,
    pub min_timer_interval: Option<i32>,
    pub priority: Option<XEnginePriority>,
    pub max_script_event_queue: Option<i32>,
    pub thread_stack_size: Option<i32>,
    pub app_domain_loading: Option<bool>,
    pub script_stop_strategy: Option<ScriptStopStrategy>,
    pub delete_scripts_on_startup: Option<bool>,
    pub compact_mem_on_load: Option<bool>,
    pub compile_with_debug_information: Option<bool>,
    pub event_limit: Option<i32>,
    pub kill_timed_out_scripts: Option<bool>,
    pub script_delay_factor: Option<i32>,
    pub script_distance_limit_factor: Option<i32>,
    pub notecard_line_read_chars_max: Option<i32>,
    pub sensor_max_range: Option<i32>,
    pub sensor_max_results: Option<i32>,
    pub disable_underground_movement: Option<i32>,
    pub script_engines_path: Option<PathBuf>,
}
impl XEngine {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("MinThreads", &self.min_threads);
        append_line!("MaxThreads", &self.max_threads);
        append_line!("IdleTimeout", &self.idle_timeout);
        append_line!("MinTimerInterval", &self.min_timer_interval);
        append_line!("Priority", &self.priority);
        append_line!("MaxScriptEventQueue", &self.max_script_event_queue);
        append_line!("ThreadStackSize", &self.thread_stack_size);
        append_line!("AppDomainLoading", &self.app_domain_loading);
        append_line!("ScriptStopStrategy", &self.script_stop_strategy);
        append_line!("DeleteScriptsOnStartup", &self.delete_scripts_on_startup);
        append_line!("CompactMemOnLoad", &self.compact_mem_on_load);
        append_line!(
            "CompileWithDebugInformation",
            &self.compile_with_debug_information
        );
        append_line!("EventLimit", &self.event_limit);
        append_line!("KillTimedOutScripts", &self.kill_timed_out_scripts);
        append_line!("ScriptDelayFactor", &self.script_delay_factor);
        append_line!(
            "ScriptDistanceLimitFactor",
            &self.script_distance_limit_factor
        );
        append_line!(
            "NotecardLineReadCharsMax",
            &self.notecard_line_read_chars_max
        );
        append_line!("SensorMaxRange", &self.sensor_max_range);
        append_line!("SensorMaxResults", &self.sensor_max_results);
        append_line!(
            "DisableUndergroundMovement",
            &self.disable_underground_movement
        );
        append_line!(
            "ScriptEnginesPath",
            &self
                .script_engines_path
                .as_ref()
                .map(|p| p.to_string_lossy())
        );

        result
    }
}
impl Default for XEngine {
    fn default() -> Self {
        Self {
            app_domain_loading: Some(false),
            enabled: Default::default(),
            min_threads: Default::default(),
            max_threads: Default::default(),
            idle_timeout: Default::default(),
            min_timer_interval: Default::default(),
            priority: Default::default(),
            max_script_event_queue: Default::default(),
            thread_stack_size: Default::default(),
            script_stop_strategy: Default::default(),
            delete_scripts_on_startup: Default::default(),
            compact_mem_on_load: Default::default(),
            compile_with_debug_information: Default::default(),
            event_limit: Default::default(),
            kill_timed_out_scripts: Default::default(),
            script_delay_factor: Default::default(),
            script_distance_limit_factor: Default::default(),
            notecard_line_read_chars_max: Default::default(),
            sensor_max_range: Default::default(),
            sensor_max_results: Default::default(),
            disable_underground_movement: Default::default(),
            script_engines_path: Default::default(),
        }
    }
}

pub struct OSSL {
    pub include_ossl_default_enable: Option<PathBuf>,
}
impl OSSL {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!(
            "Include-osslDefaultEnable",
            &self
                .include_ossl_default_enable
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        result
    }
}
impl Default for OSSL {
    fn default() -> Self {
        Self {
            include_ossl_default_enable: Some(PathBuf::from(
                "config-include/osslDefaultEnable.ini",
            )),
        }
    }
}

#[derive(Default)]
pub struct FreeSwitchVoice {
    pub enabled: Option<bool>,
    pub local_service_module: Option<String>,
    pub free_switch_service_url: Option<String>,
}

impl FreeSwitchVoice {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("LocalServiceModule", &self.local_service_module);
        append_line!("FreeSwitchServiceURL", &self.free_switch_service_url);

        result
    }
}

#[derive(Default)]
pub struct Groups {
    pub enabled: Option<bool>,
    pub level_group_create: Option<i32>,
    pub module: Option<GroupsModule>,
    pub storage_provider: Option<String>,
    pub services_connector_module: Option<ServicesConnectorModule>,
    pub local_service: Option<LocalService>,
    pub secret_key: Option<String>,
    pub groups_server_uri: Option<String>,
    pub home_uri: Option<String>,
    pub messaging_enabled: Option<bool>,
    pub messaging_module: Option<GroupsMessagingModule>,
    pub notices_enabled: Option<bool>,
    pub message_online_users_only: Option<bool>,
    pub debug_enabled: Option<bool>,
    pub debug_messaging_enabled: Option<bool>,
    pub xml_rpc_service_read_key: Option<i32>,
    pub xml_rpc_service_write_key: Option<i32>,
}
impl Groups {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("LevelGroupCreate", &self.level_group_create);
        append_line!("Module", &self.module);
        append_line!("StorageProvider", &self.storage_provider);
        append_line!("ServicesConnectorModule", &self.services_connector_module);
        append_line!("LocalService", &self.local_service);
        append_line!("SecretKey", &self.secret_key);
        append_line!("GroupsServerUIR", &self.groups_server_uri);
        append_line!("HomeURI", &self.home_uri);
        append_line!("MessagingEnabled", &self.messaging_enabled);
        append_line!("MessagingModule", &self.messaging_module);
        append_line!("NoticesEnabled", &self.notices_enabled);
        append_line!("MessageOnlineUsersOnly", &self.message_online_users_only);
        append_line!("DebugEnabled", &self.debug_enabled);
        append_line!("DebugMessagingEnabled", &self.debug_messaging_enabled);
        append_line!("XmlRpcServiceReadKey", &self.xml_rpc_service_read_key);
        append_line!("XmlRpcServiceWriteKey", &self.xml_rpc_service_write_key);

        result
    }
}

#[derive(Default)]
pub struct InterestManagement {
    pub update_prioritization_scheme: Option<UpdatePrioritizationScheme>,
    pub objects_culling_by_distance: Option<bool>,
}
impl InterestManagement {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!(
            "UpdatePrioritizationScheme",
            &self.update_prioritization_scheme
        );
        append_line!(
            "ObjectsCullingByDistance",
            &self.objects_culling_by_distance
        );

        result
    }
}

#[derive(Default)]
pub struct MediaOnAPrim {
    pub enabled: Option<bool>,
}
impl MediaOnAPrim {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        result
    }
}

#[derive(Default)]
pub struct NPC {
    pub enabled: Option<bool>,
    pub max_number_npcs_per_scene: Option<i32>,
    pub allow_not_owned: Option<bool>,
    pub allow_sense_as_avatar: Option<bool>,
    pub allow_clone_other_avatars: Option<bool>,
    pub no_npc_group: Option<bool>,
}
impl NPC {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("MaxNumberNPCsPerScene", &self.max_number_npcs_per_scene);
        append_line!("AllowNotOwned", &self.allow_not_owned);
        append_line!("AllowSenseAsAvatar", &self.allow_sense_as_avatar);
        append_line!("AllowCloneOtherAvatars", &self.allow_clone_other_avatars);
        append_line!("NoNPCGroup", &self.no_npc_group);

        result
    }
}

#[derive(Default)]
pub struct Terrain {
    pub initial_terrain: Option<String>,
}
impl Terrain {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("InitialTerrain", &self.initial_terrain);

        result
    }
}

#[derive(Default)]
pub struct LandManagement {
    pub show_parcel_bans_lines: Option<String>,
}
impl LandManagement {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("ShowParcelBansLines", &self.show_parcel_bans_lines);

        result
    }
}

#[derive(Default)]
pub struct UserProfiles {
    pub profile_service_url: Option<String>,
    pub allow_user_profile_web_urls: Option<bool>,
}
impl UserProfiles {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("ProfileServiceURL", &self.profile_service_url);
        append_line!("AllowUserProfileWebURLs", &self.allow_user_profile_web_urls);

        result
    }
}

#[derive(Default)]
pub struct XBakes {
    pub url: Option<String>,
}
impl XBakes {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("URL", &self.url);

        result
    }
}

#[derive(Default)]
pub struct GodNames {
    pub enabled: Option<bool>,
    pub full_names: Option<GodFullNamesList>,
    pub surnames: Option<GodSurnamesList>,
}
impl GodNames {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Enabled", &self.enabled);
        append_line!("FullNames", &self.full_names);
        append_line!("Surnames", &self.surnames);

        result
    }
}

pub struct Architecture {
    pub include_architecture: Option<Architectures>,
}
impl Architecture {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }

        append_line!("Include-Architecture", &self.include_architecture);

        result
    }
}
impl Default for Architecture {
    fn default() -> Self {
        Self {
            include_architecture: Some(Architectures::Standalone),
        }
    }
}
