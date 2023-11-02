clear && docker run \
    -p 8888:80 \
    -v `pwd`/http-lfi.php:/var/www/html/index.php \
    -v `pwd`/http-php-auth.conf:/etc/nginx/conf.d/default.conf \
    trafex/php-nginx