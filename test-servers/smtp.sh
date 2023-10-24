 docker run \
    -p 2525:25 \
    -e maildomain=example.com \
    -e smtp_user=admin666:test12345 \
    catatnight/postfix