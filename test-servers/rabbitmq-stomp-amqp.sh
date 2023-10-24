docker run -it \
    --name rabbitmq \
    -p 61613:61613/tcp \
    -p 5672:5672/tcp \
    -e RABBITMQ_ADMIN_USER=admin666 \
    -e RABBITMQ_ADMIN_PASSWORD=test12345 \
    -e RABBITMQ_API_USER=user666 \
    -e RABBITMQ_API_PASSWORD=test12345 \
    thinkco/rabbitmq