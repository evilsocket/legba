---
title: STOMP brute-force (ActiveMQ, RabbitMQ)
description: Brute-force STOMP authentication on ActiveMQ, RabbitMQ, HornetQ, and OpenMQ message brokers.
---

The STOMP text protocol allows interaction with message queueing services like ActiveMQ, RabbitMQ, HornetQ and OpenMQ.

## Examples

```sh
legba stomp \
    --target localhost:61613 \
    --username admin \
    --password data/passwords.txt
```
