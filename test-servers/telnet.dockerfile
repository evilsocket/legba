FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update -y && apt install -y sudo telnetd vim systemctl && apt-get clean
RUN adduser -gecos --disabled-password --shell /bin/bash admin666
RUN echo "admin666:test12345" | chpasswd
EXPOSE 23
CMD systemctl start inetd; while [ true ]; do sleep 60; done