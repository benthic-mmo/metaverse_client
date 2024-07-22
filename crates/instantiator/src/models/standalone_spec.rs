use std::fmt;
use std::path::PathBuf;


static DEFAULT_BASE_HOSTNAME: &str = "127.0.0.1";
static DEFAULT_PUBLIC_PORT: i32 = 9000;

#[derive(Default)]
pub struct StandaloneConfig {
    pub database_service: DatabaseService,
    pub hypergrid: Hypergrid,
    pub modules: Modules,
    pub asset_service: AssetService,
    pub grid_service: GridService,
    pub library_module: LibraryModule,
    pub login_service: LoginService,
    pub free_switch_service: FreeSwitchService,
    pub grid_info_service: GridInfoService,
    pub map_image_service: MapImageService,
    pub authorization_service: AuthorizationService,
    pub gatekeeper_service: GatekeeperService,
    pub user_agent_service: UserAgentService,
    pub hg_asset_service: HgAssetService,
    pub hg_inventory_access_module: HgInventoryAccessModule,
    pub hg_friends_module: HgFriendsModule,
    pub messaging: Messaging,
    pub entity_transfer: EntityTransfer,
    pub user_profile_service: UserProfileService,
}
impl fmt::Display for StandaloneConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                result.push_str(&format!("[{}]\n{}\n", $key, $value));
            };
        }

        append_line!("DatabaseService", self.database_service.to_string());
        append_line!("Hypergrid", self.hypergrid.to_string());
        append_line!("Modules", self.modules.to_string());
        append_line!("AssetService", self.asset_service.to_string());
        append_line!("GridService", self.grid_service.to_string());
        append_line!("LibraryModule", self.library_module.to_string());
        append_line!("LoginService", self.login_service.to_string());
        append_line!("FreeSwitchService", self.free_switch_service.to_string());
        append_line!("GridInfoService", self.grid_info_service.to_string());
        append_line!("MapImageService", self.map_image_service.to_string());
        append_line!(
            "AuthorizationService",
            self.authorization_service.to_string()
        );
        append_line!("GatekeeperService", self.gatekeeper_service.to_string());
        append_line!("UserAgentService", self.user_agent_service.to_string());
        append_line!("HGAssetService", self.hg_asset_service.to_string());
        append_line!(
            "HGInventoryAccessModule",
            self.hg_inventory_access_module.to_string()
        );
        append_line!("HgFriendsModule", self.hg_friends_module.to_string());
        append_line!("Messaging", self.messaging.to_string());
        append_line!("EntityTransfer", self.entity_transfer.to_string());
        append_line!("UserProfileService", self.user_profile_service.to_string());
        write!(f, "{}", result)
    }
}

pub enum AssetType {
    Unknown,
    Texture,
    Sound,
    CallingCard,
    Landmark,
    Clothing,
    Object,
    Notecard,
    LSLText,
    LSLBytecode,
    TextureTGA,
    Bodypart,
    SoundWAV,
    ImageTGA,
    ImageJPEG,
    Animation,
    Gesture,
    Mesh,
}
impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unknown => "Unknown".to_string(),
            Self::Texture => "Texture".to_string(),
            Self::Sound => "Sound".to_string(),
            Self::CallingCard => "CallingCard".to_string(),
            Self::Landmark => "Landmark".to_string(),
            Self::Clothing => "Clothing".to_string(),
            Self::Object => "Object".to_string(),
            Self::Notecard => "Notecard".to_string(),
            Self::LSLText => "LSLText".to_string(),
            Self::LSLBytecode => "LSLBytecode".to_string(),
            Self::TextureTGA => "TextureTGA".to_string(),
            Self::Bodypart => "BodyPart".to_string(),
            Self::SoundWAV => "SoundWAV".to_string(),
            Self::ImageTGA => "ImageTGA".to_string(),
            Self::ImageJPEG => "ImageJPEG".to_string(),
            Self::Animation => "Animation".to_string(),
            Self::Gesture => "Gesture".to_string(),
            Self::Mesh => "Mesh".to_string(),
        };
    write!(f, "{}", s) 
    }
}

pub struct UsernameList(pub Vec<String>);
impl fmt::Display for UsernameList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct AssetTypeList(pub Vec<AssetType>);
impl fmt::Display for AssetTypeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}, ", region));
        }
        write!(f, "{}", output)
    }
}

pub struct RemoteGridBanList(pub Vec<String>);
impl fmt::Display for RemoteGridBanList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}
pub struct RemoteGridAcceptList(pub Vec<String>);
impl fmt::Display for RemoteGridAcceptList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct AllowedClientsList(pub Vec<String>);
impl fmt::Display for AllowedClientsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct DeniedClientsList(pub Vec<String>);
impl fmt::Display for DeniedClientsList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

pub struct RegionProperties(Vec<Region>);
impl fmt::Display for RegionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}\n", region));
        }
        write!(f, "{}", output)
    }
}

pub struct ForeignTripsAllowedLevel {
    pub level: i32,
    pub allow: bool,
}
impl fmt::Display for ForeignTripsAllowedLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForeignTripsAllowed_Level_{} = {}", self.level, self.allow)
    }
}
pub struct ForeignTripsAllowedLevelList(pub Vec<ForeignTripsAllowedLevel>);
impl fmt::Display for ForeignTripsAllowedLevelList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}\n", region));
        }
        write!(f, "{}", output)
    }
}

pub struct UrlList(pub Vec<String>);
impl fmt::Display for UrlList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

pub struct ForeignTripsDisallowedExceptLevel {
    pub level: i32,
    pub disallowed_urls: UrlList,
}
impl fmt::Display for ForeignTripsDisallowedExceptLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f,
            "DisallowExcept_Level_{} = {}",
            self.level, self.disallowed_urls) 

    }
}

pub struct ForeignTripsDisallowedExceptLevelList(pub Vec<ForeignTripsDisallowedExceptLevel>);
impl fmt::Display for ForeignTripsDisallowedExceptLevelList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}\n", region));
        }
        write!(f, "{}", output)
    }
}

pub struct ForeignTripsAllowExceptLevel {
    pub level: i32,
    pub allowed_urls: UrlList,
}
impl fmt::Display for ForeignTripsAllowExceptLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllowExcept_Level_{} = {}", self.level, self.allowed_urls)
    }
}

pub struct ForeignTripsAllowExceptLevelList(pub Vec<ForeignTripsAllowExceptLevel>);
impl fmt::Display for ForeignTripsAllowExceptLevelList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}\n", region));
        }
        write!(f, "{}", output)
    }
}

pub struct Region {
    pub name: String,
    pub properties: RegionFlags,
}
impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Region_{} = \"{}\"", self.name, self.properties)
    }
}

#[derive(Default)]
pub struct RegionFlags {
    pub default_region: Option<bool>,
    pub default_hg_region: Option<bool>,
    pub fallback_region: Option<bool>,
    pub no_direct_login: Option<bool>,
    pub persistent: Option<bool>,
    pub disallow_foreigners: Option<bool>,
    pub disallow_residents: Option<bool>,
}
impl fmt::Display for RegionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if $value.is_some() {
                    result.push_str(&format!("{},", $key));
                }
            };
        }
        append_line!("DefaultRegion", &self.default_region);
        append_line!("DefaultHGRegion", &self.default_hg_region);
        append_line!("FallbackRegion", &self.fallback_region);
        append_line!("NoDirectLogin", &self.no_direct_login);
        append_line!("Persistent", &self.persistent);
        append_line!("DisallowForeigners", &self.disallow_foreigners);
        append_line!("DisallowResidents", &self.disallow_residents);
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct DatabaseConnection {
    pub data_source: Option<String>,
    pub server: Option<String>,
    pub database: String,
    pub user_id: String,
    pub password: String,
    pub old_guids: Option<String>,
    pub ssl_mode: Option<String>,
}
impl fmt::Display for DatabaseConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value.as_ref().filter(|s| !s.is_empty()) {
                    result.push_str(&format!("{}={};", $key, value));
                }
            };
        }

        append_line!("Data Source", &self.data_source);
        append_line!("Server", &self.server);
        result.push_str(&format!("Database={};", self.database));
        result.push_str(&format!("User ID={};", self.user_id));
        result.push_str(&format!("Password={};", self.password));
        append_line!("Old Guids", &self.old_guids);
        append_line!("SslMode", &self.ssl_mode);
        write!(f, "{}", result)
    }
}

pub struct DatabaseService {
    pub include_storage: Option<PathBuf>,
    pub storage_provider: Option<PathBuf>,
    pub connection_string: Option<DatabaseConnection>,
    pub estate_connection_string: Option<DatabaseConnection>,
}
impl fmt::Display for DatabaseService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!(
            "Include-Storage",
            &self.include_storage.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "StorageProvider",
            &self.storage_provider.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!("ConnectionString", &self.connection_string);
        append_line!("EstateConnectionString", &self.estate_connection_string);
        write!(f, "{}", result)
    }
}
impl Default for DatabaseService {
    fn default() -> Self {
        Self {
            include_storage: Some(PathBuf::from("config-include/storage/SQLiteStandalone.ini")),
            storage_provider: Default::default(),
            connection_string: Default::default(),
            estate_connection_string: Default::default(),
        }
    }
}


pub struct Hypergrid {
    pub gate_keeper_uri: Option<String>,
    pub gate_keeper_uri_alias: Option<String>,
    pub home_uri: Option<String>,
    pub home_uri_alias: Option<String>,
}
impl fmt::Display for Hypergrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("GateKeeperURI", &self.gate_keeper_uri);
        append_line!("GateKeeperURIAlias", &self.gate_keeper_uri_alias);
        append_line!("HomeURI", &self.home_uri);
        append_line!("HomeURIAlias", &self.home_uri_alias);
        write!(f, "{}", result)
    }
}
impl Default for Hypergrid {
    fn default() -> Self {
        Self {
            home_uri: Some(format!("{}:{}", DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT)),
            gate_keeper_uri: Default::default(),
            gate_keeper_uri_alias: Default::default(),
            home_uri_alias: Default::default(),
        }
    }
}

pub struct Modules {
    pub asset_caching: Option<String>,
    pub include_flotsam_cache: Option<PathBuf>,
}
impl fmt::Display for Modules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("AssetCaching", &self.asset_caching);
        append_line!(
            "Include-FlotsamCache",
            &self
                .include_flotsam_cache
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        write!(f, "{}", result)
    }
}
impl Default for Modules {
    fn default() -> Self {
        Self {
            asset_caching: Some("FlotsamAssetCache".to_string()),
            include_flotsam_cache: Some(PathBuf::from("config-include/FlotsamCache.ini")),
        }
    }
}

pub struct AssetService {
    pub default_asset_loader: Option<PathBuf>,
    pub asset_loader_args: Option<PathBuf>,
}
impl fmt::Display for AssetService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!(
            "DefaultAssetLoader",
            &self
                .default_asset_loader
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!(
            "AssetLoaderArgs",
            &self.asset_loader_args.as_ref().map(|p| p.to_string_lossy())
        );
        write!(f, "{}", result)
    }
}
impl Default for AssetService {
    fn default() -> Self {
        Self {
            default_asset_loader: Some(PathBuf::from(
                "OpenSim.Framework.AssetLoader.Filesystem.dll",
            )),
            asset_loader_args: Some(PathBuf::from("assets/AssetSets.xml")),
        }
    }
}

pub struct GridService {
    pub storage_provider: Option<PathBuf>,
    pub map_tile_directory: Option<PathBuf>,
    pub region_properties: Option<RegionProperties>,
    pub export_supported: Option<bool>,
}
impl fmt::Display for GridService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    if $key == "" {
                        result.push_str(&format!("{}\n", value.to_string()));
                    } else {
                        result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()))
                    };
                }
            };
        }
        append_line!(
            "StorageProvider",
            &self.storage_provider.as_ref().map(|p| p.to_string_lossy())
        );
        append_line!(
            "MapTileDirectory",
            &self
                .map_tile_directory
                .as_ref()
                .map(|p| p.to_string_lossy())
        );
        append_line!("", &self.region_properties);
        append_line!("ExportSupported", &self.export_supported);
        write!(f, "{}", result)
    }
}
impl Default for GridService {
    fn default() -> Self {
        Self {
            storage_provider: Some(PathBuf::from("OpenSim.Data.Null.dll:NullRegionData")),
            region_properties: Some(RegionProperties(vec![Region {
                name: "Welcome_Area".to_string(),
                properties: RegionFlags {
                    default_region: Some(true),
                    default_hg_region: Some(true),
                    ..Default::default()
                },
            }])),

            export_supported: Some(true),
            map_tile_directory: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct LibraryModule {
    pub library_name: Option<String>,
}
impl fmt::Display for LibraryModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("LibraryName", &self.library_name);
        write!(f, "{}", result)
    }
}

pub struct LoginService {
    pub welcome_message: Option<String>,
    pub srv_home_uri: Option<String>,
    pub srv_inventory_server_uri: Option<String>,
    pub srv_asset_server_uri: Option<String>,
    pub srv_profile_server_uri: Option<String>,
    pub srv_friends_server_uri: Option<String>,
    pub srv_im_server_uri: Option<String>,
    pub map_tile_url: Option<String>,
    pub search_url: Option<String>,
    pub destination_guide: Option<String>,
    pub min_log_level: Option<String>,
    pub currency: Option<String>,
    pub classified_fee: Option<String>,
    pub allow_login_fallback_to_any_region: Option<i32>,
    pub dos_allow_x_forwarded_for_header: Option<bool>,
    pub dos_max_requests_in_time_frame: Option<i32>,
    pub dos_forgive_client_after_ms: Option<i32>,
}
impl fmt::Display for LoginService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("WelcomeMessage", &self.welcome_message);
        append_line!("SRV_HomeURI", &self.srv_home_uri);
        append_line!("SRV_InventoryServerURI", &self.srv_inventory_server_uri);
        append_line!("SRV_AssetServerURI", &self.srv_asset_server_uri);
        append_line!("SRV_ProfileServerURI", &self.srv_profile_server_uri);
        append_line!("SRV_FriendsServerURI", &self.srv_friends_server_uri);
        append_line!("SRV_IMServerURI", &self.srv_im_server_uri);
        append_line!("MapTileURL", &self.map_tile_url);
        append_line!("SearchURL", &self.search_url);
        append_line!("DestinationGuide", &self.destination_guide);
        append_line!("MinLogLevel", &self.min_log_level);
        append_line!("Currency", &self.currency);
        append_line!("ClassifiedFee", &self.classified_fee);
        append_line!(
            "AllowLoginFallbackToAnyRegion",
            &self.allow_login_fallback_to_any_region
        );
        append_line!(
            "DOSAllowXForwardedForHeader",
            &self.dos_allow_x_forwarded_for_header
        );
        append_line!(
            "DOSMaxRequestsInTimeFrame",
            &self.dos_max_requests_in_time_frame
        );
        append_line!("DOSForgiveClientAfterMS", &self.dos_forgive_client_after_ms);
        write!(f, "{}", result)
    }
}
impl Default for LoginService {
    fn default() -> Self {
        Self {
            welcome_message: Some("Welcome, Avatar!".to_string()),
            srv_home_uri: Some(format!("{}:{}", DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT)),
            srv_inventory_server_uri: Some(format!(
                "{}:{}",
                DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT
            )),
            srv_asset_server_uri: Some(format!(
                "{}:{}",
                DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT
            )),
            srv_profile_server_uri: Some(format!(
                "{}:{}",
                DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT
            )),
            srv_friends_server_uri: Some(format!(
                "{}:{}",
                DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT
            )),
            srv_im_server_uri: Some(format!("{}:{}", DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT)),
            map_tile_url: Some(format!(
                "{}:{}/",
                DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT
            )),
            search_url: Default::default(),
            destination_guide: Default::default(),
            min_log_level: Default::default(),
            currency: Default::default(),
            classified_fee: Default::default(),
            allow_login_fallback_to_any_region: Default::default(),
            dos_allow_x_forwarded_for_header: Default::default(),
            dos_max_requests_in_time_frame: Default::default(),
            dos_forgive_client_after_ms: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct FreeSwitchService {
    pub server_address: Option<String>,
    pub realm: Option<String>,
    pub sip_proxy: Option<String>,
    pub default_timeout: Option<i32>,
    pub context: Option<String>,
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub echo_server: Option<String>,
    pub echo_port: Option<i32>,
    pub attempt_stun: Option<bool>,
}
impl fmt::Display for FreeSwitchService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("ServerAddress", &self.server_address);
        append_line!("Realm", &self.server_address);
        append_line!("SIPProxy", &self.sip_proxy);
        append_line!("DefaultTimeout", &self.default_timeout);
        append_line!("Context", &self.context);
        append_line!("UserName", &self.user_name);
        append_line!("Password", &self.password);
        append_line!("EchoServer", &self.echo_server);
        append_line!("EchoPort", &self.echo_port);
        append_line!("AttemptSTUN", &self.attempt_stun);
        write!(f, "{}", result)
    }
}

pub struct GridInfoService {
    pub login: Option<String>,
    pub grid_name: Option<String>,
    pub grid_nick: Option<String>,
    pub welcome: Option<String>,
    pub economy: Option<String>,
    pub about: Option<String>,
    pub register: Option<String>,
    pub help: Option<String>,
    pub password: Option<String>,
    pub gatekeeper: Option<String>,
    pub uas: Option<String>,
    pub grid_status: Option<String>,
    pub grid_status_rss: Option<String>,
}
impl fmt::Display for GridInfoService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("login", &self.login);
        append_line!("gridname", &self.grid_name);
        append_line!("gridnick", &self.grid_nick);
        append_line!("welcome", &self.welcome);
        append_line!("economy", &self.economy);
        append_line!("about", &self.about);
        append_line!("register", &self.register);
        append_line!("help", &self.help);
        append_line!("password", &self.password);
        append_line!("gatekeeper", &self.gatekeeper);
        append_line!("uas", &self.uas);
        append_line!("GridStatus", &self.grid_status);
        append_line!("GridStatusRSS", &self.grid_status_rss);

        write!(f, "{}", result)
    }
}
impl Default for GridInfoService {
    fn default() -> Self {
        Self {
            login: Some(format!("{}:{}", DEFAULT_BASE_HOSTNAME, DEFAULT_PUBLIC_PORT)),
            grid_name: Some("the lost continent of hippo".to_string()),
            grid_nick: Some("hippogrid".to_string()),
            welcome: Default::default(),
            economy: Default::default(),
            about: Default::default(),
            register: Default::default(),
            help: Default::default(),
            password: Default::default(),
            gatekeeper: Default::default(),
            uas: Default::default(),
            grid_status: Default::default(),
            grid_status_rss: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct MapImageService {
    pub tile_storage_path: Option<String>,
}
impl fmt::Display for MapImageService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("TileStoragePath", &self.tile_storage_path);
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct AuthorizationService {
    pub region_properties: Option<RegionProperties>,
}
impl fmt::Display for AuthorizationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self.region_properties {
            Some(value) => value.to_string(),
            None => "".to_string(),

        };
        write!(f, "{}", s)
    }
}

pub struct GatekeeperService {
    pub external_name: Option<String>,
    pub allow_teleports_to_any_region: Option<bool>,
    pub allowed_clients: Option<AllowedClientsList>,
    pub denied_clients: Option<DeniedClientsList>,
    pub foreign_agents_allowed: Option<bool>,
    pub allow_except: Option<RemoteGridBanList>,
    pub disallow_except: Option<RemoteGridAcceptList>,
}
impl fmt::Display for GatekeeperService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("ExternalName", &self.external_name);
        append_line!(
            "AllowTeleportsToAnyRegion",
            &self.allow_teleports_to_any_region
        );
        append_line!("AllowedClients", &self.allowed_clients);
        append_line!("ForeignAgentsAllowed", &self.foreign_agents_allowed);
        append_line!("AllowExcept", &self.allow_except);
        append_line!("DisallowExcept", &self.disallow_except);
        write!(f, "{}", result)
    }
}
impl Default for GatekeeperService {
    fn default() -> Self {
        Self {
            external_name: Default::default(),
            allow_teleports_to_any_region: Some(true),
            allowed_clients: Default::default(),
            denied_clients: Default::default(),
            foreign_agents_allowed: Default::default(),
            allow_except: Default::default(),
            disallow_except: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct UserAgentService {
    pub level_outside_contacts: Option<i32>,
    pub foreign_trips_allowed_level: Option<ForeignTripsAllowedLevelList>,
    pub foreign_trips_disallowed_except_level: Option<ForeignTripsDisallowedExceptLevelList>,
    pub foreign_trips_allowed_except: Option<ForeignTripsAllowedLevelList>,
    pub show_user_details_in_hg_profile: Option<bool>,
}
impl fmt::Display for UserAgentService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    if $key == "" {
                        result.push_str(&format!("{}\n", value.to_string()));
                    } else {
                        result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                    }
                }
            };
        }
        append_line!("LevelOutsideContacts", &self.level_outside_contacts);
        append_line!("", &self.foreign_trips_allowed_level);
        append_line!("", &self.foreign_trips_allowed_except);
        append_line!("", &self.foreign_trips_disallowed_except_level);
        append_line!(
            "ShowUserDetailsInHGProfile",
            &self.show_user_details_in_hg_profile
        );
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct HgAssetService {
    pub disallow_export: Option<AssetTypeList>,
    pub disallow_import: Option<AssetTypeList>,
}
impl fmt::Display for HgAssetService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("DisallowExport", &self.disallow_export);
        append_line!("DisallowImport", &self.disallow_import);
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct HgInventoryAccessModule {
    pub outbound_permission: Option<bool>,
    pub restrict_inventory_access_abroad: Option<bool>,
}
impl fmt::Display for HgInventoryAccessModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("OutboundPermission", &self.outbound_permission);
        append_line!(
            "RestrictInventoryAccessAbroad",
            &self.restrict_inventory_access_abroad
        );
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct HgFriendsModule {
    pub level_hg_friends: Option<i32>,
}
impl fmt::Display for HgFriendsModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("LevelHgFriends", &self.level_hg_friends);
        write!(f, "{}", result)
    }
}

#[derive(Default)]
pub struct Messaging {}
impl fmt::Display for Messaging {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

pub struct EntityTransfer {
    pub level_hg_teleport: Option<i32>,
    pub restrict_appearance_abroad: Option<bool>,
    pub account_for_appearance: Option<UsernameList>,
}
impl fmt::Display for EntityTransfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("LevelHGTeleport", &self.level_hg_teleport);
        append_line!("RestrictAppearanceAbroad", &self.restrict_appearance_abroad);
        append_line!("AccountForAppearance", &self.account_for_appearance);
        write!(f, "{}", result)
    }
}
impl Default for EntityTransfer {
    fn default() -> Self {
        Self {
            level_hg_teleport: Default::default(),
            restrict_appearance_abroad: Default::default(),
            account_for_appearance: Some(UsernameList(vec![
                "Test User".to_string(),
                "Astronaut Smith".to_string(),
            ])),
        }
    }
}

pub struct UserProfileService {
    pub enabled: Option<bool>,
    pub local_service_module: Option<String>,
    pub connection_string: Option<DatabaseConnection>,
    pub realm: Option<String>,
    pub user_account_service: Option<String>,
    pub authentication_service_module: Option<String>,
}
impl fmt::Display for UserProfileService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        macro_rules! append_line {
            ($key:expr, $value:expr) => {
                if let Some(value) = $value {
                    result.push_str(&format!("{} = \"{}\"\n", $key, value.to_string()));
                }
            };
        }
        append_line!("Enabled", &self.enabled);
        append_line!("LocalServiceModule", &self.local_service_module);
        append_line!("ConnectionString", &self.connection_string);
        append_line!("Realm", &self.realm);
        append_line!("UseraAccountService", &self.user_account_service);
        append_line!(
            "AuthenticationServiceModule",
            &self.authentication_service_module
        );
write!(f, "{}", result)
    }
}
impl Default for UserProfileService {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            local_service_module: Some(
                "OpenSim.Services.UserProfilesService.dll:UserProfilesService".to_string(),
            ),
            connection_string: Default::default(),
            realm: Default::default(),
            user_account_service: Some(
                "OpenSim.Services.UserAccountService.dll:UserAccountService".to_string(),
            ),
            authentication_service_module: Some(
                "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
                    .to_string(),
            ),
        }
    }
}
