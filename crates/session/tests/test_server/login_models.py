from enum import Enum

class FriendRights(Enum): 
    NONE = 0, 
    CANSEEONLINE = 1, 
    CANSEEONMAP = 2, 
    CANMODIFYOBJECTS = 4

class classifiedCategory(object): 
    def __init__(self,
            category_id= 1,
            category_name=  "Shopping"):
        self.category_id = category_id 
        self.category_name = category_name
    def asdict(self):
        return{
                "category_id": self.category_id,
                "category_name": self.category_name}

class buddy(object): 
    def __init__(self, 
            buddy_id = "04c259b7-94bc-4822-b099-745191ffc247", 
            buddy_rights_given= 1, 
            buddy_rights_has=   1): 
        self.buddy_id=             buddy_id 
        self.buddy_rights_given=   buddy_rights_given
        self.buddy_rights_has=     buddy_rights_has
    def asdict(self):
        return {
            "buddy_id": self.buddy_id,
            "buddy_rights_given": self.buddy_rights_given,
            "buddy_rights_has": self.buddy_rights_has
            }

class gesture(object): 
    def __init__(self, 
            item_id = "004d663b-9980-46ae-8559-bb60e9d67d28" ,
            asset_id = "004d663b-9980-46ae-8559-bb60e9d67d28"): 
        self.item_id=   item_id 
        self.asset_id=  asset_id
    def asdict(self): 
        return{
            "item_id": self.item_id,
            "asset_id": self.asset_id,
            }

class initialOutfit(object): 
    def __init__(self,
            folder_name = "Nightclub Female",
            gender= "female"): 
        self.folder_name = folder_name
        self.gender = gender
    
    def asdict(self): 
        return {
            "folder_name": self.folder_name, 
            "gender" :self.gender
                }


class globalTexturesParams(object): 
    def __init__(self, 
            cloud_texture_id=   "dc4b9f0b-d008-45c6-96a4-01dd947ac621", 
            sun_texture_id=     "cce0f112-878f-4586-a2e2-a8f104bba271", 
            moon_texture_id=    "ec4b9f0b-d008-45c6-96a4-01dd947ac621"):

        self.cloud_texture_id=  cloud_texture_id
        self.sun_texture_id=    sun_texture_id
        self.moon_texture_id=   moon_texture_id
    def asdict(self): 
        return{
            "cloud_texture_id": self.cloud_texture_id,
            "sun_texture_id":   self.sun_texture_id,
            "moon_texture_id":  self.moon_texture_id
                }

class loginFlagsParams(object): 
    def __init__(self, 
            stipend_since_login=    "N", 
            ever_logged_in=         "N", 
            seconds_since_epoch=    1411075065, 
            daylight_savings=       "N", 
            gendered=               "N"): 
        self.stipend_since_login=   stipend_since_login
        self.ever_logged_in=        ever_logged_in
        self.seconds_since_epoch=   seconds_since_epoch
        self.daylight_savings=      daylight_savings 
        self.gendered=              gendered 
    def asdict(self): 
        return{
            "stipend_since_login":  self.stipend_since_login,
            "ever_logged_in":       self.ever_logged_in, 
            "seconds_since_epoch":  self.seconds_since_epoch, 
            "daylight_savings":     self.daylight_savings,
            "gendered":             self.gendered
                }

class uiConfigParams(object):
    def __init__(self, 
            allow_first_life= "Y"):
        self.allow_first_life = allow_first_life

    def asdict(self):
        return{"allow_first_life": self.allow_first_life}

class homeParams(object): 
    def __init__(self, 
            x_grid_coord            ="0", 
            y_grid_coord            ="0", 
            x_region_coord          ="0", 
            y_region_coord          ="0",
            z_region_coord          ="0", 
            x_coord                 ="0", 
            y_coord                 ="0", 
            z_coord                 ="0"): 
        self.x_grid_coord=      "r" + x_grid_coord
        self.y_grid_coord=      "r" + y_grid_coord
        self.x_region_coord=    "r" + x_region_coord 
        self.y_region_coord=    "r" + y_region_coord
        self.z_region_coord=    "r" + z_region_coord
        self.x_coord=           "r" + x_coord
        self.y_coord=           "r" + y_coord
        self.z_coord=           "r" + z_coord 

    def asdict(self): 
        return{
                "region_handle" :[self.x_grid_coord, self.y_grid_coord],
                "position"      :[self.x_region_coord, self.y_region_coord, self.z_region_coord],
                "look_at"       :[self.x_coord, self.y_coord, self.z_coord]
                }

class lookAtParams(object): 
    def __init__(self, 
            x = "0", 
            y = "0",
            z = "0"):

        self.x = "r" + x 
        self.y = "r" + y 
        self.z = "r" + z
    def aslist(self): 
        return[self.x, self.y, self.z]

class inventoryRootParams(object): 
    def __init__(self, 
            folder_id= "37c4cfe3-ea39-4ef7-bda3-bee73bd46d95"): 
        self.folder_id=     folder_id
    def asdict(self): 
        return{
            "folder_id": self.folder_id,
                }


class inventorySkeletonParams(object): 
    def __init__(self, 
            folder_id       ="004d663b-9980-46ae-8559-bb60e9d67d28",
            parent_id       ="5cb09cb9-5080-4bf4-8ba0-86b6197fcc74",
            name            ="Camera Test", 
            type_default    =-1,
            version         =2): 
        self.folder_id=     folder_id 
        self.parent_id=     parent_id 
        self.name=          name 
        self.type_default=  type_default
        self.version=       version 
    def asdict(self):
        return{
                "folder_id":    self.folder_id,
                "parent_id":    self.parent_id, 
                "name":         self.name, 
                "type_default": self.type_default, 
                "version":      self.version
                }

class inventoryLibOwner(object): 
    def __init__(self, 
            agent_id = "11111111-1111-0000-0000-000100bba000"):
        self.agent_id = agent_id
    def asdict(self):
        return{
            "agent_id": self.agent_id
                }

class loginResponse(object): 
    def __init__(self,
            home                    =homeParams().asdict(), 
            look_at                 =lookAtParams().aslist(), 
            agent_access            ="M", 
            agent_access_max        ="A",
            seed_capability         ="http://192.168.1.2:9000",
            first_name              ="First", 
            last_name               ="Last",
            agent_id                ="11111111-1111-0000-0000-000100bba000",
            sim_ip                  ="192.168.1.2",
            sim_port                =9000,
            http_port               =0,
            start_location          ="last",
            region_x                =256000, 
            region_y                =256000,
            region_size_x           =256,
            region_size_y           =256,
            circuit_code            =697482820,
            session_id              ="6ac2e761-f490-4122-bf6c-7ad8fbb17002",
            secure_session_id       ="fe210274-9056-467a-aff7-d95f60bacccc",
            inventory_root          =inventoryRootParams().asdict(),
            inventory_skeleton      =[inventorySkeletonParams().asdict(), inventorySkeletonParams().asdict()],
            inventory_lib_root      =inventoryRootParams().asdict(),
            inventory_skel_lib      =[inventorySkeletonParams().asdict(), inventorySkeletonParams().asdict()], 
            inventory_lib_owner     =inventoryLibOwner().asdict(), 
            map_server_url          ="http://192.168.1.2:8002/", 
            buddy_list              =[buddy().asdict(), buddy().asdict(), buddy().asdict()],
            gestures                =[gesture().asdict(), gesture().asdict()],
            initial_outfit          =initialOutfit().asdict(),
            global_textures         =globalTexturesParams().asdict(),
            login                   ="true", 
            login_flags             =loginFlagsParams().asdict(),
            message                 ="Welcome, Avatar!",
            ui_config               =uiConfigParams().asdict(),
            event_categories        =[],
            classified_categories   =[classifiedCategory().asdict(), classifiedCategory().asdict()]
            ):
        self.home=                  home
        self.look_at=               look_at 
        self.agent_access=          agent_access
        self.agent_access_max=      agent_access_max
        self.seed_capability=       seed_capability
        self.first_name=            first_name 
        self.last_name=             last_name 
        self.agent_id=              agent_id 
        self.sim_ip=                sim_ip
        self.sim_port=              sim_port
        self.http_port=             http_port
        self.start_location=        start_location 
        self.region_x=              region_x 
        self.region_y=              region_y 
        self.region_size_x=         region_size_x
        self.region_size_y=         region_size_y
        self.circuit_code=          circuit_code
        self.session_id=            session_id 
        self.secure_session_id=     secure_session_id
        self.inventory_root=        inventory_root 
        self.inventory_skeleton=    inventory_skeleton
        self.inventory_lib_root=    inventory_lib_root
        self.inventory_skel_lib=    inventory_skel_lib
        self.inventory_lib_owner=   inventory_lib_owner 
        self.map_server_url=        map_server_url
        self.buddy_list=            buddy_list
        self.gestures=              gestures 
        self.initial_outfit=        initial_outfit 
        self.global_textures=       global_textures 
        self.login=                 login 
        self.login_flags=           login_flags 
        self.message=               message 
        self.ui_config=             ui_config 
        self.event_categories=      event_categories 
        self.classified_categories= classified_categories 
    def asdict(self):
        return{"home":                      self.home,
               "look_at":                   self.look_at,
               "agent_access":              self.agent_access,
               "agent_access_max":          self.agent_access_max, 
               "seed_capability":           self.seed_capability, 
               "first_name":                self.first_name, 
               "last_name":                 self.last_name, 
               "agent_id":                  self.agent_id, 
               "sim_ip":                    self.sim_ip, 
               "sim_port":                  self.sim_port,
               "http_port":                 self.http_port,
               "start_location":            self.start_location, 
               "region_x":                  self.region_x, 
               "region_y":                  self.region_y, 
               "region_size_x":             self.region_size_x, 
               "region_size_y":             self.region_size_y, 
               "circuit_code":              self.circuit_code, 
               "session_id":                self.session_id, 
               "secure_session_id":         self.secure_session_id,
               "inventory-root":            self.inventory_root, 
               "inventory-skeleton":        self.inventory_skeleton,
                "inventory-lib-root":       self.inventory_lib_root, 
                "inventory-skel-lib":       self.inventory_skel_lib, 
                "inventory-lib-owner":      self.inventory_lib_owner, 
                "map-server-url":           self.map_server_url, 
                "buddy-list":               self.buddy_list,
                "gestures":                 self.gestures, 
                "initial-outfit":           self.initial_outfit,
                "global-textures":          self.global_textures, 
                "login":                    self.login, 
                "login-flags":              self.login_flags, 
                "message":                  self.message, 
                "ui-config":                self.ui_config, 
                "event_categories":         self.event_categories,
                "classified_categories":   self.classified_categories, 
               }
    
