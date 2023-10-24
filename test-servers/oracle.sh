docker run -it \
    -p 1521:1521 \
    -e ORACLE_PASSWORD=test12345 \
    -e APP_USER=admin666 \
    -e APP_USER_PASSWORD=test12345 \
     gvenzl/oracle-xe
