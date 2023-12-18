docker run \
    --security-opt no-new-privileges \
    --name socks5 \
    --restart unless-stopped \
    -p 1080:1080 \
    -e PROXY_USER=admin666 \
    -e PROXY_PASSWORD=test12345 \
    yarmak/socks5-server