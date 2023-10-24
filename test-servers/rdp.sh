# creds: ubuntu:ubuntu
#docker run  \
#    -p 3389:3389 \
#    scottyhardy/docker-remote-desktop:latest

# creds: abc:abc
docker run \
  --name=rdesktop \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=Etc/UTC \
  -p 3389:3389 \
  --restart unless-stopped \
  lscr.io/linuxserver/rdesktop:latest