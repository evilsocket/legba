FROM alpine:3.18

# Install dropbear and necessary tools
RUN apk add --no-cache \
    dropbear \
    dropbear-scp \
    dropbear-ssh \
    openssh-keygen \
    bash

# Create SSH directory
RUN mkdir -p /etc/dropbear

# Generate host keys (RSA and DSS for legacy support)
RUN dropbearkey -t rsa -f /etc/dropbear/dropbear_rsa_host_key -s 2048
RUN dropbearkey -t dss -f /etc/dropbear/dropbear_dss_host_key

# Set root password (change this in production!)
RUN echo 'root:legacy123' | chpasswd

# Create a startup script to run dropbear with legacy options
RUN echo '#!/bin/sh' > /start-dropbear.sh && \
    echo 'exec dropbear -F -E -s -g -j -k -r /etc/dropbear/dropbear_rsa_host_key -r /etc/dropbear/dropbear_dss_host_key -p 22' >> /start-dropbear.sh && \
    chmod +x /start-dropbear.sh

# Expose SSH port
EXPOSE 22

# Start dropbear
CMD ["/start-dropbear.sh"]