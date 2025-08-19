MQTT password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--mqtt-client-id <MQTT_CLIENT_ID>` | MQTT client identifier [default: `legba`] |
| `--mqtt-v5` | Use MQTT v5 |

## Examples

```sh
legba mqtt \
    --target 127.0.0.1:1883 \
    --username admin \
    --password wordlists/passwords.txt \
```
