The command (cmd) plugin allows legba to interact with a custom executable and use either its exit code or a string pattern to determine a success or failure. It can be used to integrate with clients and utilities that are not natively supported by legba and parallelize their execution in order to attack credentials.

## Options

| Name | Description |
| ---- | ----------- |
| `--cmd-binary <CMD_BINARY>` | Command binary [default: not set]  |
| `--cmd-args <CMD_ARGS>` | Command arguments. {USERNAME}, {PASSWORD}, {TARGET} and {PORT} can be used as placeholders [default: not set] |
| `--cmd-success-exit-code <CMD_SUCCESS_EXIT_CODE>` | Process exit code to be considered as a positive match [default: `0`] |
| `--cmd-success-match <CMD_SUCCESS_MATCH>` | String to look for in the process standard output to be considered as a positive match |

## Examples

Use the unzip utility to find the password of a password protected ZIP archive (as seen in [this recipe](https://github.com/evilsocket/legba-cookbook/tree/main/zip)):

```sh
legba cmd \
    --single-match \
    --cmd-binary unzip \
    --cmd-args "\\-oP '{PASSWORD}' \\-d /tmp {TARGET}" \
    -U "" \
    --password wordlist.txt \
    --target data/protected.zip
```