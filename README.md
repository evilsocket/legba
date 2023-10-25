`Legba` is a multiprotocol credentials bruteforcer / password sprayer and enumerator built with Rust and the Tokio asynchronous runtime in order to achieve
better performances and stability while consuming less resources than similar tools.

**Work in progress:** while the tool is functioning well overall, it still requires some testing and the integration of more protocols. If you want to contribute with code and/or testing, feel free to check the list of TODOs with `grep -ri --include "*.rs" TODO` ^_^

Currently supported protocols / plugins (use `legba --list-plugins` to print this list):

| Plugin Name  | Description |
| ------------- | ------------- |
| amqp       | AMQP password authentication (ActiveMQ, RabbitMQ, Qpid, JORAM and Solace). |
| dns        | DNS subdomain enumeration. |
| ftp        | FTP password authentication. |
| http       | HTTP request for custom web logins supporting CSRF. |
| http.basic | HTTP basic authentication. |
| http.enum  | Web pages enumeration. |
| http.form  | HTTP multipart form request. |
| http.ntlm1 | NTLMv1 authentication over HTTP. |
| http.ntlm2 | NTLMv2 authentication over HTTP. |
| imap       | IMAP password authentication. |
| kerberos   | Kerberos 5 (pre)authentication and users enumeration. |
| ldap       | LDAP password authentication. |
| mongodb    | MongoDB password authentication. |
| mssql      | Microsoft SQL Server password authentication. |
| mysql      | MySQL password authentication. |
| pgsql      | PostgreSQL password authentication. |
| pop3       | POP3 password authentication. |
| rdp        | Microsoft Remote Desktop password authentication. |
| sftp       | SFTP password and key based authentication. |
| smtp       | SMTP password authentication. |
| ssh        | SSH password and key based authentication. |
| stomp      | STOMP password authentication (ActiveMQ, RabbitMQ, HornetQ and OpenMQ). |
| telnet     | Telnet password authentication. |
| vnc        | VNC password authentication. |

## Building From Sources

Building the project from sources requires [Rust to be installed](https://rustup.rs/):

```sh
cargo build --release
```

The binary will be compiled inside the `./target/release` folder.

## Docker Image

Alternatively it is possible to build a Docker container:

```sh
docker build -t legba .
```

And then run it via:

```sh
docker run legba --help # or any other command line
```

## Usage

The tool requires a plugin name, a `--target` argument specifying the ip, hostname and (optionally) the port of the target and, depending on the selected plugin, a pair of `--username` and `--password` arguments or a single `--data` argument (like in the case of the `dns.enum` plugin which requires a single enumeration element).

The `--username`/`--data` and `--password`/`--key` arguments all support the same logic depending on the value passed to them:

* If the value provided is an existing file name, it'll be loaded as a wordlist.
* If the value provided is in the form of `@/some/path/*.txt` it'll be used as a [glob expression](https://docs.rs/glob/latest/glob/) to iterate matching files.
* If the value provided is in the form of `#<NUMBER>-<NUMBER>:<OPTIONAL CHARSET>`, it'll be used to generate all possible permutations of the given charset (or the default one if not provided) and of the given length. For instance: `#1-3` will generate all permutations from 1 to 3 characters using the default ASCII printable charset, while `#4-5:0123456789` will generate all permutations of digits of 4 and 5 characters.
* Anything else will be considered as a constant string.

For instance:

* `legba <plugin name> --username admin --password data/passwords.txt` will always use `admin` as username while loading the passwords from a wordlist.
* `legba <plugin name> --username data/users.txt --password data/passwords.txt` will load both from wordlists and use all combinations.
* `legba <plugin name> --username admin` will always use `admin` as username and attempt all permutations of the default printable ASCII charset between 4 and 8 characters (this is the default behaviour when a value is not passed).
* `legba <plugin name> --username data/users.txt --password '@/some/path/*.key'` will load users from a wordlist while testing all key files inside `/some/path`.
* `legba <plugin name> --username data/users.txt --password '#4-5:abcdef'` will load users from a wordlist while testing all permutations of the charaters `abcdef` 4 and 5 characters long.

### Main Options

| Option | Default | Description |
| ------ | ------- | ----------- |
| `--list-plugins` | | List all available protocol plugins and exit. |
| `-t, --target <TARGET>` | | Target host, url or IP address with optional port if different than the protocol default. |
| `--data, --username <USERNAME>` | `#4-8` | Constant, filename, glob expression as `@/some/path/*.txt` or range as `#min-max:charset` / `#min-max`. |
| `--key, --password <USERNAME>` | `#4-8` | Constant, filename, glob expression as `@/some/path/*.txt` or range as `#min-max:charset` / `#min-max`. |
| `-s, --session <FILENAME>` | | Save and restore session information from this file. |
| `-o, --output <OUTPUT>` | | Save results to this file. |
| `--output-format <FORMAT>` | `text` | Output file format [possible values: text, jsonl] |
| `--timeout <TIMEOUT>` | `1000` | Connection timeout in milliseconds. |
| `--retries <RETRIES>` | `10` | Number of attempts if a request fails. |
| `--retry-time <TIME>` | `1000` | Delay in milliseconds to wait before a retry. |
| `--single-match` | |  Exit after the first positive match is found. | 
| `--ulimit <ULIMIT>` | `10000` | Value for ulimit (max open file descriptors). | 
| `--concurrency <VALUE>` | `10` |  Number of concurrent workers. |
| `--rate-limit <LIMIT>` | `0` | Limit the number of requests per second. |
| `--jitter-min <VALUE>` | `0` | Minimum number of milliseconds for random request jittering. |
| `--jitter-max <VALUE>` | `0` | Maximum number of milliseconds for random request jittering. |
| `--quiet` | | Do not report statistics. |
| `-h, --help` | | Print help. |
| `-V, --version` | | Print version. |

For the full list of arguments including plugin specific ones run `legba --help`.

### Examples

**NOTE:** The port in the `--target` argument is optional whenever it matches the default port for the given protocol.

#### HTTP Basic Authentication

```sh
legba http.basic \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/
```

#### HTTP Post Request (Wordpress wp-login.php page):

```sh
legba http \
    --username admin \
    --password wordlists/passwords.txt \
    --target http://localhost:8888/wp-login.php \
    --http-method POST \
    --http-success-codes 302 \ # wordpress redirects on successful login
    --http-payload 'log={USERNAME}&pwd={PASSWORD}'
```

#### HTTP Post Request (Wordpress xmlrpc.php)

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

#### HTTP Post Request with CSRF Token grabbing:

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

#### HTTP Request with NTLMv1 Authentication:

```sh
legba http.ntlm1 \
    --domain example.org \
    --workstation client \
    --username admin \
    --password wordlists/passwords.txt \
    --target https://localhost:8888/
```

#### HTTP Request with NTLMv2 Authentication:

```sh
legba http.ntlm2 \
    --domain example.org \
    --workstation client \
    --username admin \
    --password wordlists/passwords.txt \
    --target https://localhost:8888/
```

#### HTTP Pages Enumeration:
 
```sh
legba http.enum \
    --data data/pages.txt \
    --target http://localhost:8888/ \
    --http-enum-ext php \ # php is the default value for file extensions
    --http-success-codes 200 
```

#### DNS Subdomain Enumeration:

```sh
legba dns \
    --data data/200k-dns.txt \
    --target something.com \
    --dns-resolvers "1.1.1.1" # comma separated list of DNS resolvers, do not pass to use the system resolver
```

#### SSH:

Password based authentication:

```sh
legba ssh \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:22
```

Key based authentication, testing keys inside /some/path:

```sh
legba ssh \
    --username admin \
    --password '@/some/path/*' \
    --ssh-auth-mode key \
    --target localhost:22
```

#### SFTP:

Password based authentication:

```sh
legba sftp \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:22
```

Key based authentication, testing keys inside /some/path:

```sh
legba sftp \
    --username admin \
    --password '@/some/path/*' \
    --ssh-auth-mode key \
    --target localhost:22
```

#### FTP Password Authentication:

```sh
legba ftp \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:21
```

#### Telnet Password Authentication:

```sh
legba telnet \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:23 \
    --telnet-user-prompt "login: " \
    --telnet-pass-prompt "Password: " \
    --telnet-prompt ":~$ " \
    --single-match # this option will stop the program when the first valid pair of credentials will be found, can be used with any plugin
```

#### SMTP Password Authentication:

```sh
legba smtp \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:25
```

#### POP3 Password Authentication:

Insecure:

```sh
legba pop3 \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:110
```

Via SSL:

```sh
legba pop3 \
    --username admin@example.com \
    --password wordlists/passwords.txt \
    --target localhost:995 \
    --pop3-ssl
```

#### MySQL Password Authentication:

```sh
legba mysql \
    --username root \
    --password wordlists/passwords.txt \
    --target localhost:3306
```

#### Microsoft SQL Server Password Authentication:

```sh
legba mssql \
    --username SA \
    --password wordlists/passwords.txt \
    --target localhost:1433
```

#### PostgresSQL Password Authentication:

```sh
legba pgsql \
    --username admin \
    --password wordlists/passwords.txt \
    --target localhost:5432  
```

#### Oracle Password Authentication

**NOTE**: this is an optional feature that is not compiled by default, enable during compilation with by using `cargo build --release -F oracle`.

```sh
legba oracle \
    --target localhost:1521 \
    --oracle-database SYSTEM \
    --username admin \
    --password data/passwords.txt
```

#### LDAP Password Authentication:

```sh
legba ldap \
    --target 127.0.0.1:389 \
    --username admin \
    --password @wordlists/passwords.txt \
    --ldap-domain example.org \
    --single-match
```

#### Kerberos 5 Pre Auth (users enumeration and password authentication):

**NOTE:** due to the way that the realm string is uppercase'd in order to generate the cryptographic salt for Microsoft domain controllers, you'll need to add the `--kerberos-linux` argument when targeting Linux Kerberos servers.

```sh
legba kerberos \
    --target 127.0.0.1 \
    --username admin \
    --password wordlists/passwords.txt \
    --kerberos-realm example.org
```

#### VNC Password Authentication:

```sh
legba vnc \
    --target localhost:5901 \
    --password data/passwords.txt
```

#### STOMP Password Authentication:

The STOMP text protocol allows interaction with message queueing services like ActiveMQ, RabbitMQ, HornetQ and OpenMQ.

```sh
legba stomp \
    --target localhost:61613 \
    --username admin \
    --password data/passwords.txt
```

#### AMQP Password Authentication:

The AMQP binary protocol allows interaction with message queueing services like ActiveMQ, RabbitMQ, Qpid, JORAM and Solace.

```sh
legba amqp \
    --target localhost:5672 \
    --username admin \
    --password data/passwords.txt
```

## License

Legba was made with ♥  by [Simone Margaritelli](https://www.evilsocket.net/) and it's released under the GPL 3 license.

To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.