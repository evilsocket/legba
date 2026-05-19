#!/usr/bin/env bash
# Regenerates docs/llms-full.txt by concatenating all docs pages in nav order.
# Run from repo root. Invoked by .github/workflows/docs.yml before mkdocs build.
set -euo pipefail

cd "$(dirname "$0")/.."

OUT="docs/llms-full.txt"
BASE_URL="https://legba.evilsocket.net"

{
  printf '# legba — full documentation\n\n'
  printf 'Single-file concatenation of every page under https://legba.evilsocket.net/, intended for ingestion by LLMs and AI agents. Source: https://github.com/evilsocket/legba/tree/main/docs\n\n'
  printf 'Generated: %s\n\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf -- '---\n\n'
} > "$OUT"

emit() {
  local path="$1"
  local relpath="${path#docs/}"
  local url_path="${relpath%.md}"
  if [[ "$url_path" == "index" ]]; then
    url="$BASE_URL/"
  else
    url="$BASE_URL/$url_path/"
  fi
  {
    printf '\n\n## Source: %s\n\n' "$url"
    cat "$path"
    printf '\n\n---\n'
  } >> "$OUT"
}

# Order mirrors mkdocs.yml nav.
pages=(
  docs/index.md
  docs/install.md
  docs/usage.md
  docs/recipes.md
  docs/rest.md
  docs/mcp.md
  docs/benchmark.md
  docs/comparison.md
  docs/faq.md
  docs/plugins/http.md
  docs/plugins/ssh_and_sftp.md
  docs/plugins/ftp.md
  docs/plugins/smtp.md
  docs/plugins/imap.md
  docs/plugins/pop3.md
  docs/plugins/rdp.md
  docs/plugins/vnc.md
  docs/plugins/samba.md
  docs/plugins/ldap.md
  docs/plugins/kerberos.md
  docs/plugins/mysql.md
  docs/plugins/postgresql.md
  docs/plugins/mssql.md
  docs/plugins/oracle.md
  docs/plugins/mongodb.md
  docs/plugins/scylla.md
  docs/plugins/redis.md
  docs/plugins/amqp.md
  docs/plugins/mqtt.md
  docs/plugins/stomp.md
  docs/plugins/snmp.md
  docs/plugins/irc.md
  docs/plugins/telnet.md
  docs/plugins/dns.md
  docs/plugins/port_scanner.md
  docs/plugins/socks5.md
  docs/plugins/custom_binary.md
)

for p in "${pages[@]}"; do
  if [[ -f "$p" ]]; then
    emit "$p"
  else
    echo "warn: $p missing, skipping" >&2
  fi
done

echo "wrote $OUT ($(wc -l < "$OUT") lines, $(wc -c < "$OUT") bytes)"
