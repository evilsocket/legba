LDAP Password Authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--ldap-domain <LDAP_DOMAIN>` | LDAP domain |

## Examples

```sh
legba ldap \
    --target 127.0.0.1:389 \
    --username admin \
    --password @wordlists/passwords.txt \
    --ldap-domain example.org \
    --single-match
```