IRC server password authentication.

## Options

| Name | Description |
| ---- | ----------- |
| `--irc-tls` | Use TLS for IRC [default: `false`] |

## Examples

IRC password authentication:

```sh
legba irc \
    --password wordlists/passwords.txt \
    --target irc.example.com:6667
```

IRC password authentication with TLS:

```sh
legba irc \
    --password wordlists/passwords.txt \
    --irc-tls \
    --target irc.example.com:6697
```

