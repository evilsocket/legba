FROM nginx:alpine
COPY http-basic-auth.html /usr/share/nginx/html/index.html
COPY http-basic-auth.conf /etc/nginx/nginx.conf
COPY http-basic-auth.htpasswd /etc/nginx/.htpasswd