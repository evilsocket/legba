docker build -t basic-auth -f http-basic-auth.dockerfile . && \
clear &&
docker run -p 8888:80 basic-auth