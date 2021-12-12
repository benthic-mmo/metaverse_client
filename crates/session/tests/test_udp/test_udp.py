import socket 

IP = "127.0.0.1" 
PORT = 20001
BUFFER = 1024

sock = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)
sock.bind((IP, PORT))

print("UDP server up and listening on " + IP + ":" + str(PORT)) 

while(True):
    bytesAddressPair = sock.recvfrom(BUFFER)
    message = bytesAddressPair[0]
    address = bytesAddressPair[1]
    clientMsg = "Message from Client:{}".format(message) 
    clientIP = "Client IP Address:{}".format(address) 
    print(clientMsg) 
    print(clientIP)
    sock.sendto(str.encode("aAAAAAAAAAAAA"), address) 
