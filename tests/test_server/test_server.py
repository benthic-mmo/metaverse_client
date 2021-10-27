# this is the test server for the login module
# it's just a basic python xmlrpc server.

import login_models
from xmlrpc.server import SimpleXMLRPCServer 
from xmlrpc.server import SimpleXMLRPCRequestHandler
from enum import Enum



PORT = 8000
HOST = '127.0.0.1'

class RequestHandler(SimpleXMLRPCRequestHandler): 
    rpc_paths = ('/',) 

'''
        xmlData["address_size"]
        xmlData["agree_to_tos"]
        xmlData["channel"]
        xmlData["first"]
        xmlData["id0"]
        xmlData["last"]
        xmlData["last_exec_duration"]
        xmlData["last_exec_event"]
        xmlData["mac"]
        xmlData["options"]
        xmlData["options"]["adult_compliant"]
        xmlData["options"]["advanced_mode"]
        xmlData["options"]["avatar_picker_url"]
        xmlData["options"]["buddy-list"]
        xmlData["options"]["classified_categories"]
        xmlData["options"]["currency"]
        xmlData["options"]["destination_guide_url"]
        xmlData["options"]["display_names"]
        xmlData["options"]["event_categories"]
        xmlData["options"]["gestures"]
        xmlData["options"]["global-textures"] 
        xmlData["options"]["inventory-lib-root"]
        xmlData["options"]["inventory-root"]
        xmlData["options"]["inventory-skel-lib"]
        xmlData["options"]["inventory-skeleton"]
        xmlData["options"]["login-flags"]
        xmlData["options"]["map-server-url"]
        xmlData["options"]["max-agent-groups"]
        xmlData["options"]["max_groups"]
        xmlData["options"]["newuser-config"]
        xmlData["options"]["search"]
        xmlData["options"]["tutorial_setting"]
        xmlData["options"]["ui-config"]
        xmlData["options"]["voice-config"]
        xmlData["passwd"]
        xmlData["platform"]
        xmlData["platform_string"]
        xmlData["platform_version"]
        xmlData["read_critical"]
        xmlData["skipoptional"]
        xmlData["start"]
        xmlData["version"]
        xmlData["viewer_digest"]
'''

##TODO: generate mock login response'
## defined here http://opensimulator.org/wiki/SimulatorLoginProtocol 
with SimpleXMLRPCServer((HOST, PORT), requestHandler = RequestHandler) as server: 
    def login_to_simulator(xmlData): 
        response = login_models.loginResponse()
        return response 
    server.register_function(login_to_simulator)

    # prints where it's serving on for debug
    print("serving on: " + HOST + ":" + str(PORT))
    server.serve_forever()
