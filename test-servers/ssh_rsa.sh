docker run \
  --name=openssh-server \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=Etc/UTC \
  -e SUDO_ACCESS=true \
  -e PASSWORD_ACCESS=false \
  -e USER_PASSWORD=test12345 \
  -e USER_NAME=admin666 \
  -v $(pwd)/ssh_rsa_config:/etc/ssh/sshd_config \
  -p 2222:2222 \
  --rm \
  lscr.io/linuxserver/openssh-server:latest