# REST API

Legba has a REST API that can be activated by using the `--api address:port` command line argument.

To start the API (it is recommended to always bind it to localhost) on a given port:

```sh
legba --api 127.0.0.1:8080
```

To set which origins are allowed:

```sh
legba --api 127.0.0.1:8080 --api-allowed-origin 127.0.0.1:1234
```

To allow any origin (use this at your own risk):

```sh
legba --api 127.0.0.1:8080 --api-allowed-origin any
```

## Routes

### GET `localhost:8080/api/plugins`

Returns a list of available plugins and their options:

```json
[
    {
        "name": "amqp",
        "description": "AMQP password authentication (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace).",
        "strategy": "username_and_password",
        "options": {
            "amqp_ssl": {
                "name": "amqp_ssl",
                "description": "Enable SSL for AMQP",
                "value": false
            }
        },
        "override_payload": null
    },
    ... etc etc ...
]
```

### POST `localhost:8080/api/session/new`

POSTs an array of command line arguments to start a new Legba session.

#### Request

```json
[
    "http",
    "-T",
    "localhost",
    "-U", "admin", 
    "-P", "admin"
]
```

#### Response

The new session identifier:

```
54e54b44-db39-4b1d-819a-dd12926a59bf
```

### GET `localhost:8080/api/session/<session id>`

Returns a session status given its identifier:

```json
{
    "id": "54e54b44-db39-4b1d-819a-dd12926a59bf",
    "plugin_name": "http",
    "targets": [
        "localhost"
    ],
    "process_id": 45178,
    "client": "127.0.0.1:64829",
    "argv": [
        "http",
        "-T",
        "localhost",
        "-U",
        "admin",
        "-P",
        "admin"
    ],
    "started_at": 1734528859,
    "statistics": {
        "tasks": 12,
        "memory": "24.8 MiB",
        "targets": 1,
        "attempts": 1,
        "errors": 0,
        "done": 0,
        "done_percent": 0.0,
        "reqs_per_sec": 0
    },
    "loot": [],
    "output": [
        "legba v0.10.0",
        "[INFO ] target: localhost",
        "[INFO ] username -> string 'admin'",
        "[INFO ] password -> string 'admin'",
        "[ERROR] [localhost] attempt 5/5: error sending request for url (http://localhost/): error trying to connect: tcp connect error: Connection refused (os error 61)",
        "[INFO ] runtime 5.009478792s"
    ],
    "completed": {
        "completed_at": 1734528864,
        "exit_code": 0,
        "error": null
    }
}
```

### GET `localhost:8080/api/session/<session id>/stop`

Stops a session given its identifier.

### GET `localhost:8080/api/sessions`

List all available sessions.