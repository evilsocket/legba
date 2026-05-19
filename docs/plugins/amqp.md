---
title: AMQP brute-force plugin (ActiveMQ, RabbitMQ)
description: Brute-force AMQP authentication on ActiveMQ, RabbitMQ, Qpid, JORAM, and Solace brokers with legba's async Rust engine.
---

The AMQP binary protocol allows interaction with message queueing services like ActiveMQ, RabbitMQ, Qpid, JORAM and Solace.

## Options

| Name | Description |
| ---- | ----------- | 
| `--amqp-ssl` | Enable SSL for AMQP. |

## Examples

```sh
legba amqp \
    --target localhost:5672 \
    --username admin \
    --password data/passwords.txt
```