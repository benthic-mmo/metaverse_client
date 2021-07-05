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
    def Login(x, y): 
        return x + y
    server.register_function(Login)

    # prints where it's serving on for debug
    print("serving on: " + HOST + ":" + str(PORT))
    server.serve_forever()
