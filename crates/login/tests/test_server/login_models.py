from enum import Enum

class FriendRights(Enum): 
    NONE = 0, 
    CANSEEONLINE = 1, 
    CANSEEONMAP = 2, 
    CANMODIFYOBJECTS = 4

class classifiedCategory(object): 
    def __init__(self,
            number= "None",
            label=  "None"):
        self.number = number, 
        self.label = label
    def asdict(self):
        return{number, label}

class buddy(object): 
    def __init__(self, 
            buddy_id, 
            buddy_rights_given= FriendRights.NONE, 
            buddy_rights_has=   FriendRights.NONE): 
        self.buddy_id=             buddy_id 
        self.buddy_rights_given=   buddy_rights_given
        self.buddy_rights_has=     buddy_rights_has

class gesture(object): 
    def __init__(self, 
            item_id,
            asset_id): 
        self.item_id=   item_id 
        self.asset_id=  asset_id

class globalTexturesParams(object): 
    def __init__(self, 
            cloud_texture_id=   "None", 
            sun_texture_id=     "None", 
            moon_texture_id=    "None"):

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
            stipend_since_login=    "None", 
            ever_logged_in=         "None", 
            seconds_since_epoch=   "None", 
            daylight_savings=       "None", 
            gendered=               "None"): 
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
            allow_first_life= "None"):
        self.allow_first_life = allow_first_life

    def asdict(self):
        return{"allow_first_life": self.allow_first_life}

class homeParams(object): 
    def __init__(self, 
            x_grid_coord            ="None", 
            y_grid_coord            ="None", 
            x_region_coord          ="None", 
            y_region_coord          ="None",
            z_region_coord          ="None", 
            x_coord                 ="None", 
            y_coord                 ="None", 
            z_coord                 ="None"): 
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
            x = "None", 
            y = "None",
            z = "None"):

        self.x = "r" + x 
        self.y = "r" + y 
        self.z = "r" + z
    def aslist(self): 
        return[self.x, self.y, self.z]

class inventorySkeletonParams(object): 
    def __init__(self, 
            folder_id       ="None",
            parent_id       ="None",
            name            ="None", 
            type_default    ="None",
            version         ="None"): 
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


class loginResponse(object): 
    def __init__(self,
            home                    =homeParams().asdict(), 
            look_at                 =lookAtParams().aslist(), 
            agent_access            ="None", 
            agent_access_max        ="None",
            seed_capability         ="None",
            first_name              ="None", 
            last_name               ="None",
            agent_id                ="None",
            sim_ip                  ="None",
            sim_port                ="None",
            http_port               ="None",
            start_location          ="None",
            region_x                ="None", 
            region_y                ="None",
            region_size_x           ="None",
            region_size_y           ="None",
            circuit_code            ="None",
            session_id              ="None",
            secure_session_id       ="None",
            inventory_root          ="None",
            inventory_skeleton      =inventorySkeletonParams().asdict(),
            inventory_lib_root      ="None",
            inventory_skel_lib      ="None", 
            inventory_lib_owner     ="None", 
            map_server_url          ="None", 
            buddy_list              =[],
            gestures                =[],
            initial_outfit          ="None",
            global_textures         =globalTexturesParams().asdict(),
            login                   ="None", 
            login_flags             =loginFlagsParams().asdict(),
            message                 ="None",
            ui_config               =uiConfigParams().asdict(),
            event_categories        ="None",
            classified_categories   =[]
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
                "classified_categoories":   self.classified_categories 
               }
    
