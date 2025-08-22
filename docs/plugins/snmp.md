# SNMP

SNMP (Simple Network Management Protocol) community and credential enumeration with OID tree discovery.

Legba supports three SNMP protocol versions:
- **SNMPv1/v2**: Community string enumeration
- **SNMPv3**: Username and password authentication with automatic protocol detection

## Protocol Details

### SNMPv1/v2
These versions use community strings for authentication. The plugin will enumerate valid community strings and retrieve available OIDs (Object Identifiers) from the target device.

### SNMPv3
SNMPv3 provides enhanced security with username/password authentication. The plugin automatically attempts multiple authentication protocols:
- MD5
- SHA1
- SHA224
- SHA256
- SHA384
- SHA512

When a valid credential is found, the plugin will enumerate all accessible OIDs and their values.

## Examples

Test common community strings against an SNMPv1 device:

```sh
legba snmp1 \
    --payload wordlists/snmp-communities.txt \
    # a short 50ms timeout is recommended for LAN targets
    --timeout 50 \
    --target 192.168.1.1
```

Same but against a whole subnet:

```sh
legba snmp1 \
    --payload wordlists/snmp-communities.txt \
    --timeout 50 \
    --target 192.168.1.0/24
```

Walk the entire SNMP tree:

```sh
legba snmp1 \
    --payload wordlists/snmp-communities.txt \
    # a short 50ms timeout is recommended for LAN targets
    --timeout 50 \
    # removes the default limit
    --snmp-max 0 \
    --target 192.168.1.1
```

Read a single OID instead of walking the entire tree:

```sh
legba snmp1 \
    --payload wordlists/snmp-communities.txt \
    # a short 50ms timeout is recommended for LAN targets
    --timeout 50 \
    --snmp-oid '1.3.6.1.2.1.1' \
    --target 192.168.1.1
```

Test community strings against an SNMPv2 device:

```sh
legba snmp2 \
    --payload public,private,manager \
    --timeout 50 \
    --target 192.168.1.1:161
```

Test username/password combinations with automatic protocol detection:

```sh
legba snmp3 \
    --username admin \
    --password wordlists/passwords.txt \
    --timeout 50 \
    --target 10.0.0.1
```

Test multiple users and passwords:

```sh
legba snmp3 \
    --username admin,snmpuser,monitor \
    --password wordlists/top-passwords.txt \
    --timeout 50 \
    --target snmp.example.com
```