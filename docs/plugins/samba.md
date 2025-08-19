Samba username and password authentication.

## Options

| Name | Description |
| ---- | ----------- | 
| `--smb-workgroup <SMB_WORKGROUP>` | Samba workgroup name [default: `WORKGROUP`] |
| `--smb-share <SMB_SHARE>` | Expicitly set Samba private share to test. |

## Examples

Will try to autodetect a private share to test:

```sh
legba smb \
    --target share.company.com \
    --username admin \
    --password data/passwords.txt
```

Pass private share by hand:


```sh
legba smb \
    --target share.company.com \
    --username admin \
    --password data/passwords.txt \
    --smb-share "/private_share"
```
