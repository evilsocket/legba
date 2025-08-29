TCP and UDP port scanner with http banner grabbing capabilities.

## Options

| Name | Description |
| ---- | ----------- |
| `--port-scanner-ports <PORT_SCANNER_PORTS>` |  Range or comma separated values of integer port numbers to scan [default to most common ports] |
| `--port-scanner-no-banners` |  Do not attempt banner grabbing |
| `--port-scanner-no-tcp` |  Do not perform TCP scan |
| `--port-scanner-no-udp` |  Do not perform UDP scan |
| `--port-scanner-banner-timeout <PORT_SCANNER_BANNER_TIMEOUT>` |  Timeout in seconds for banner grabbing [default: `1000`] |
| `--port-scanner-http <PORT_SCANNER_HTTP>` | Comma separated list of ports for HTTP grabbing [default: `"80, 8080, 8081, 8888"`] |
| `--port-scanner-https <PORT_SCANNER_HTTPS>` | Comma separated list of ports for HTTPS grabbing [default: `"443, 8443"`] |
| `--port-scanner-http-headers <PORT_SCANNER_HTTP_HEADERS>` | Comma separated list lowercase header names for HTTP/HTTPS grabbing [default: `"server, x-powered-by, location"`] |

## Examples

Scan all TCP and UDP ports with a 300ms timeout:

```sh
legba port.scanner \
    --target something.com \
    --timeout 300 
```

Scan a custom range of ports with a 300ms timeout:

```sh
legba port.scanner \
    --target something.com \
    --port-scanner-ports '[80-10000]' \ # it's important to use the '[start-stop]' syntax to indicate a port range
    --timeout 300 
```

Scan a custom list of ports with a 300ms timeout:

```sh
legba port.scanner \
    --target something.com \
    --port-scanner-ports '21, 22, 80, 443, 8080' \
    --timeout 300 
```