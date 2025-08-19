DNS subdomain enumeration.

## Options

| Name | Description |
| ---- | ----------- |
| `--dns-resolvers <DNS_RESOLVERS>` | Comma separatd list of DNS resolvers to use instead of the system one. |
| `--dns-port <DNS_PORT>` | Resolver(s) port [default: `53`] |
| `--dns-attempts <DNS_ATTEMPTS>` | Number of retries after lookup failure before giving up [default: `1`] |

## Examples

```sh
legba dns \
    --payloads data/200k-dns.txt \
    --target something.com \
    --dns-resolvers "1.1.1.1" # comma separated list of DNS resolvers, do not pass to use the system resolver
```