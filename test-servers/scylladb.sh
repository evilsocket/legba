docker run -it --name scylladb \
    -p 9042:9042 \
    scylladb/scylla --authenticator PasswordAuthenticator
