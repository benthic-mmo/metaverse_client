# this is just for testing manually if the example server is running
import xmlrpc.client 


with xmlrpc.client.ServerProxy("http://127.0.0.1:8000/") as proxy: 
    fo = open("rawxml", "r+")
    rawxml = fo.readline();
    print(str(proxy.login_to_simulator(rawxml)))
    fo.close()
