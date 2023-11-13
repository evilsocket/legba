clear && docker run \
    -p 8888:80 \
    -v `pwd`/http-vhost:/var/www/secret \
    -v `pwd`/http-vhost.html:/var/www/html/index.html \
    -v `pwd`/http-vhost-public.conf:/etc/nginx/conf.d/public.conf \
    -v `pwd`/http-vhost-secret.conf:/etc/nginx/conf.d/secret.conf \
    trafex/php-nginx