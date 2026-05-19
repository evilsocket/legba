## Version 1.3.0 (2026-05-19)

### 🚀 New Features
- **Documentation site at https://legba.evilsocket.net/** — mkdocs-material site with full-text search, comparison page (legba vs Hydra, Medusa, Ncrack, Patator), FAQ, JSON-LD/schema.org structured data, sitemap, and `llms.txt` / `llms-full.txt` for LLM ingestion.
- **Samba shares enumeration** — new `smb.shares` plugin (closes #86).
- **Agent-driven release protocol** — `pkg/release.py` replaced by `AGENTS.md` / `CLAUDE.md`, with pre-flight checks, docs audit, and lockstep version sync.
- Rate limiting and `--wait` delays moved from the blocking credential iterator to the async dispatch loop, improving responsiveness and stop-signal handling.
- `cmd` plugin now uses `tokio::process` instead of blocking `std::process`.

### 🐛 Fixes
- HTTP success expressions no longer error when `set_cookie` is referenced but absent from the response (closes #93).
- Fixed panic when `--timeout` is combined with `-R/--recipe` due to a clap type mismatch (fixes #95).
- MySQL plugin now correctly handles usernames and passwords containing special characters by using `MySqlConnectOptions` instead of string interpolation (fixes #96).
- HTTP request errors now surface the underlying cause (e.g. "operation timed out") and a hint when the timeout is the root cause (ref #88).
- HTTP authenticated strategies (basic, NTLMv1, NTLMv2) skip success-code validation, fixing false negatives (fixes #84).
- User-provided payloads now correctly take precedence over plugin default overrides.

### 📚 Documentation
- README links to the new docs site at `legba.evilsocket.net`.
- Several documentation fixes and additions including the comparison page, FAQ, and per-page SEO/GEO metadata.

### Miscellaneous
- Homebrew formula version bump.
- Test server for HTTP plugin switched to MariaDB.
- Plugin manager unit tests now serialize access to the global `INVENTORY`, eliminating a pre-existing parallel-test flake.
- Various clippy / lint-driven refactors.

## Version 1.2.0 (2025-09-12)

### 🚀 New Features
- **Adaptive timeout system** - Timeout-sensitive plugins like DNS and port scanner can now dynamically adjust worker timeouts for better performance
- **Port scanner improvements** - Enhanced banner grabbing and protocol detection, now defaults to scanning common ports instead of full 1-65535 range
- **Performance optimizations** - Precompiled HTTP success expressions, dedicated DNS resolver objects per worker, and configurable report intervals (--report-time)
- **MCP server enhancements** - Improved prompts for better clarity about plugins and tooling

### 🐛 Fixes
- Fixed default regexp for HTTP CSRF token name
- Fixed parsing of multiple comma-separated credential expressions
- Fixed VNC plugin password field naming and reduced log verbosity (#82)
- Ensured DNS plugin only uses host targets (removes schema, port, etc.)
- Restored original default value for --http-follow-redirects

### 📚 Documentation
- Added Bludit CMS example (#83)
- Fixed CSRF regex documentation for HTTP plugin

### Miscellaneous
- Improved DNS resolver memory allocations
- Replaced HashMaps with DashMap/DashSet in DNS plugin for better performance
- Added TCP_NODELAY and single HTTP client for port scanner
- Updated MCP tools to return string responses for increased compatibility
- Various CI improvements and minor refactoring
- Added human coded badge
- Homebrew formula version bump

## Version 1.1.1 (2025-08-22)

### New Features 🚀
- Improved SNMP plugin with full SNMP tree walking capabilities
- Added project as a Homebrew tap for easier installation on macOS
- Enhanced release script and deployment process
- Added insecure TLS configuration option for MQTT

### Fixes 🐛
- Fixed Debian package generation to build with MUSL
- Resolved crates.io publishing issue

### Other
- Improved Debian package metadata
- Various small fixes and general refactoring improvements

## Version 1.1.0 (2025-08-21)

### New Features 🚀
- **Pure Rust Dependencies**: Replaced MQTT and SMB dependencies with pure Rust crates for easier cross-compilation
- **SNMP Support**: Added SNMP v1, v2 and v3 plugin support
- **HTTP Improvements**: 
  - New `--http-success` boolean expression mechanism for better success/failure detection
  - HTTP plugin now follows redirects by default
- **SSL/TLS Support**: Added SSL/TLS support for MQTT connections with `--mqtt-ssl` option
- **MCP Server Enhancements**: Now supports stdio mode as well as SSE
- **Cross-Compilation**: Legba can now be cross-compiled for any platform (native dependency free)
- **JSON Output**: Added `-J/--json` argument to print loot and statistics as JSON lines
- **Dynamic Placeholders**: Added `{user}` placeholder replacement in password templates

### Fixes 🔧
- Fixed HTTP hostname interpolation handling
- Fixed TLS 'Bad Protocol Version' errors in HTTP plugin
- Fixed MongoDB empty credentials handling
- Fixed Redis authentication detection when no auth required
- Fixed SQL authentication success detection without database permissions
- Fixed Ctrl-C signal handling for immediate stop
- Fixed Windows compilation errors

### Documentation 📚
- Moved documentation from GitHub wiki to standalone markdown files
- Added Azure, Firebase/GCP and AWS enumeration examples
- Added session save/restore and output format documentation
- Updated Samba and IRC plugin documentation

### Miscellaneous
- Optimized worker distribution logic for performance improvements
- Updated multiple dependencies to latest versions
- Added GitHub release action and improved CI/CD pipeline
- Added Android testing via cross tool
- Refactored REST API for faster session data parsing
- Various small fixes and general refactoring