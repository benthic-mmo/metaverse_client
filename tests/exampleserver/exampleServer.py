from xmlrpc.server import SimpleXMLRPCServer 
from xmlrpc.server import SimpleXMLRPCRequestHandler
PORT = 8000
HOST = '127.0.0.1'

class RequestHandler(SimpleXMLRPCRequestHandler): 
    rpc_paths = ('/',) 

with SimpleXMLRPCServer((HOST, PORT), 
        requestHandler = RequestHandler) as server: 

    def Login(x, y): 
        return x + y
    server.register_function(Login)

    print("serving on: " + HOST + ":" + str(PORT))
    server.serve_forever()
