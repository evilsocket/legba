clear && docker run \
    -p 8888:80 \
    -v `pwd`/http-xss.php:/var/www/html/index.php \
    -v `pwd`/http-nginx.conf:/etc/nginx/conf.d/default.conf \
    trafex/php-nginx

