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