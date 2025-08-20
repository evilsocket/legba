# SNMP

SNMP (Simple Network Management Protocol) community and credential enumeration with OID discovery.

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

### SNMPv1 Community Enumeration

Test common community strings against an SNMPv1 device:

```sh
legba snmp1 \
    --payload wordlists/snmp-communities.txt \
    --target 192.168.1.1
```

### SNMPv2 Community Enumeration

Test community strings against an SNMPv2 device:

```sh
legba snmp2 \
    --payload public,private,manager \
    --target 192.168.1.1:161
```

### SNMPv3 Authentication

Test username/password combinations with automatic protocol detection:

```sh
legba snmp3 \
    --username admin \
    --password wordlists/passwords.txt \
    --target 10.0.0.1
```

Test multiple users and passwords:

```sh
legba snmp3 \
    --username admin,snmpuser,monitor \
    --password wordlists/top-passwords.txt \
    --target snmp.example.com
```