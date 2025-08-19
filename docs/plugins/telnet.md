Telnet password authentication.

## Options

| Name | Description |
| ---- | ----------- |
| `--telnet-user-prompt <TELNET_USER_PROMPT>` | Telnet server username login prompt string [default: `"login: "`] |
| `--telnet-pass-prompt <TELNET_PASS_PROMPT>` | Telnet server password login prompt string [default: `"Password: "`] |
| `--telnet-prompt <TELNET_PROMPT>` | Telnet server shell prompt after successful login [default: `":~$ "`] |

## Examples

```sh
legba telnet \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:23 \
    --telnet-user-prompt "login: " \
    --telnet-pass-prompt "Password: " \
    --telnet-prompt ":~$ " \
    --single-match # this option will stop the program when the first valid pair of credentials will be found, can be used with any plugin
```