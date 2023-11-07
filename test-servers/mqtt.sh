docker run -it \
    -p 1883:1883 \
    -p 9001:9001 \
    -v ./mosquitto.conf:/mosquitto/config/mosquitto.conf \
    -v ./mosquitto_passwd:/mosquitto/config/mosquitto/passwd \
    eclipse-mosquitto