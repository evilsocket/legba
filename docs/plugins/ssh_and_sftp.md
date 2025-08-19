SSH/SFTP password and private key authentication.

## Options

| Name | Description |
| ---- | ----------- |
| `--ssh-auth-mode <SSH_AUTH_MODE>` | Authentication strategy [default: `password`] [possible values: `key`, `password`] |
| `--ssh-key-passphrase <SSH_KEY_PASSPHRASE>` | Optional private key passphrase for key based authentication. |

## Examples


SSH password based authentication:

```sh
legba ssh \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:22
```

SSH key based authentication, testing keys inside /some/path:

```sh
legba ssh \
    --username admin \
    --password '@/some/path/*' \
    --ssh-auth-mode key \
    --target localhost:22
```

SFTP password based authentication:

```sh
legba sftp \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:22
```

SFTP ley based authentication, testing keys inside /some/path:

```sh
legba sftp \
    --username admin \
    --password '@/some/path/*' \
    --ssh-auth-mode key \
    --target localhost:22
```