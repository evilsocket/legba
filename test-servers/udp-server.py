#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import socket

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

server_address = '127.0.0.1'
server_port = 1234

server = (server_address, server_port)
sock.bind(server)
print("Listening on " + server_address + ":" + str(server_port))

while True:
    payload, client_address = sock.recvfrom(1)
    print("Echoing data back to " + str(client_address))
    sent = sock.sendto(payload, client_address)
