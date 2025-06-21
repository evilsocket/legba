#!/bin/sh

# Remove any existing stronger host keys to force RSA usage
rm -f /etc/dropbear/dropbear_dss_host_key 2>/dev/null
rm -f /etc/dropbear/dropbear_ecdsa_host_key 2>/dev/null
rm -f /etc/dropbear/dropbear_ed25519_host_key 2>/dev/null

# Ensure only RSA host key exists
if [ ! -f /etc/dropbear/dropbear_rsa_host_key ]; then
    dropbearkey -t rsa -f /etc/dropbear/dropbear_rsa_host_key -s 2048
fi

# -F: Don't fork into background (for container)
# -E: Log to stderr
# -p 22: Listen on port 22
# -K 300: Keepalive timeout
# -I 0: No idle timeout
# -s: Disable password authentication (pubkey only)
# -g: Disable password authentication for root
exec dropbear -F -E -p 22 -K 300 -I 0 -s -g