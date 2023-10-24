docker build -t telnetd -f telnet.dockerfile . && \
clear &&
docker run -p 2323:23 telnetd