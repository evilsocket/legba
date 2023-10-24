docker run \
  --name=openssh-server \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=Etc/UTC \
  -e SUDO_ACCESS=true \
  -e PASSWORD_ACCESS=true \
  -e USER_PASSWORD=test12345 \
  -e USER_NAME=admin666 \
  -p 2222:2222 \
  --restart unless-stopped \
  lscr.io/linuxserver/openssh-server:latest