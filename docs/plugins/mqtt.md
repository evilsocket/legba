MQTT password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--mqtt-client-id <MQTT_CLIENT_ID>` | MQTT client identifier [default: `legba`] |
| `--mqtt-v5` | Use MQTT v5 |
| `--mqtt-ssl` | Use SSL/TLS connection (mqtts://) with certificate verification disabled. |

## Examples

```sh
legba mqtt \
    --target 127.0.0.1:1883 \
    --username admin \
    --password wordlists/passwords.txt \
```
