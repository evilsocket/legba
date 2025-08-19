SOCKS5 username and password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--socks5-address <SOCKS5_ADDRESS>` | Remote address to test the proxying for [default: `ifcfg.co`] |
| `--socks5-port <SOCKS5_PORT>` | Remote port to test the proxying for [default: `80`] |

## Examples

```sh
legba socks5 \
    --target localhost:1080 \
    --username admin \
    --password data/passwords.txt
```

With alternative address:


```sh
legba socks5 \
    --target localhost:1080 \
    --username admin \
    --password data/passwords.txt \
    --socks5-address 'internal.company.com' \
    --socks5-port 8080
```
