docker run  -p 6379:6379 redis

# then connect via redis-cli and issue the following command to create a test user:
# acl setuser aaaa on >aaaa +ping
# this creates a user "aaaa" with password "aaaa" with permissions to ping the redis server
