Kerberos 5 Pre Auth (users enumeration and password authentication).

**NOTE:** due to the way that the realm string is uppercase'd in order to generate the cryptographic salt for Microsoft domain controllers, you'll need to add the `--kerberos-linux` argument when targeting Linux Kerberos servers.

## Options

| Name | Description |
| ---- | ----------- | 
| `--kerberos-realm <KERBEROS_REALM>` | Kerberos realm |
| `--kerberos-protocol <KERBEROS_PROTOCOL>` | Kerberos transport protocol [default: `tcp`] [possible values: `udp`, `tcp`] |
| `--kerberos-linux` | If targeting a Linux Kerberos5 implementation, pass this flag to preserve the realm string case |

## Examples

```sh
legba kerberos \
    --target 127.0.0.1 \
    --username admin \
    --password wordlists/passwords.txt \
    --kerberos-realm example.org
```
