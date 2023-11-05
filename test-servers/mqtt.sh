$ docker run -it -p 1883:1883 -p 9001:9001 -v mosquitto.conf:/mosquitto/config/mosquitto.conf eclipse-mosquitto

# Example mosquitto.conf config file: https://github.com/eclipse/mosquitto/blob/master/mosquitto.conf

# To setup username/password auth for mosquitto: https://mosquitto.org/documentation/authentication-methods/