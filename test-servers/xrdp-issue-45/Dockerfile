FROM debian:12-slim

RUN apt update && apt install -y xrdp tini
RUN mkdir /app
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod 755 /app/entrypoint.sh

RUN useradd admin
RUN echo admin:Password123 | chpasswd
RUN mkdir /home/admin
RUN chown admin:admin /home/admin


ENTRYPOINT ["tini", "--", "/app/entrypoint.sh"]
# ENTRYPOINT ["/bin/bash"]
