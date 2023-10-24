docker run -it --name mongodb \
    -p 27017:27017 \
    -e MONGO_INITDB_ROOT_USERNAME=root \
    -e MONGO_INITDB_ROOT_PASSWORD=test12345 \
    -e ME_CONFIG_MONGODB_ADMINUSERNAME=admin666 \
    -e ME_CONFIG_MONGODB_ADMINPASSWORD=test12345 \
    arm64v8/mongo:latest