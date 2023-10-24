docker run \
    -p 2121:21 \
    -e USERS="admin666|test12345" \
    delfer/alpine-ftp-server