# this is the test server for the login module
# it's just a basic python xmlrpc server.

import login_models
from xmlrpc.server import SimpleXMLRPCServer 
from xmlrpc.server import SimpleXMLRPCRequestHandler
from enum import Enum

PORT = 9000
HOST = '127.0.0.1'

class RequestHandler(SimpleXMLRPCRequestHandler): 
    rpc_paths = ('/',) 

try:
    with SimpleXMLRPCServer((HOST, PORT), requestHandler = RequestHandler) as server: 
        def login_to_simulator(xmlData): 
            response = login_models.loginResponse()
            return response 
        server.register_function(login_to_simulator)

    # prints where it's serving on for debug
        print("serving on: " + HOST + ":" + str(PORT))
        server.serve_forever()
except OSError: 
    print ("address already in use. Not starting another.")
