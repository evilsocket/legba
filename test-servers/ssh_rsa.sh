docker build -t ssh_rsa -f ssh_rsa.Dockerfile . && \
clear &&
docker run \
  -p 2222:22 \
  -v $(pwd)/ssh_rsa_client_key.pub:/root/.ssh/authorized_keys \
  --rm \
  ssh_rsa