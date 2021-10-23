# this is the test server for the login module
# it's just a basic python xmlrpc server.

from xmlrpc.server import SimpleXMLRPCServer 
from xmlrpc.server import SimpleXMLRPCRequestHandler
PORT = 8000
HOST = '127.0.0.1'

class RequestHandler(SimpleXMLRPCRequestHandler): 
    rpc_paths = ('/',) 

with SimpleXMLRPCServer((HOST, PORT), 
        requestHandler = RequestHandler) as server: 

    # this defines the login command.
    # Doesn't do much right now
    # just adds to test if it's working
    def login_to_simulator(x): 
        return 1
    server.register_function(login_to_simulator)

    # prints where it's serving on for debug
    print("serving on: " + HOST + ":" + str(PORT))
    server.serve_forever()
