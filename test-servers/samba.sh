docker run -it \
    -p 139:139 \
    -p 445:445 \
    dperson/samba \
    -u "admin666;test12345" \
    -s "public;/share" \
    -s "users;/srv;no;no;no;admin666" \
    -s "admin666 private share;/admin666;no;no;no;admin666"