docker run \
    -p 5432:5432 \
	--name pgsql \
    -e POSTGRES_USER=admin666 \
	-e POSTGRES_PASSWORD=test12345 \
	postgres