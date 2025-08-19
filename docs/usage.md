# Usage

In order to use this tool, you'll need to provide:

1. A plugin name, depending on which protocol you are attacking.
2. A `--target` argument specifying the ip, hostname and (optionally) the port of the target
3. depending on the selected plugin, a pair of `--username` and `--password` arguments , a single `--payloads` argument (like in the case of the `dns.enum` plugin which requires a single enumeration element) or a single `-C/--combinations` argument.

For instance, to perform a simple HTTP basic authentication wordlist attack:

```bash
legba http.basic \
    --username admin \
    --password /path/to/wordlists.txt \
    --target https://example.com/
```

For plugins that accept a single payload, like subdomain enumeration:

```bash
legba dns \
    --payload /path/to/subdomains.txt \
    --target example.com
```

And so on.

## Selecting One or More Targets

The `--target/-T` argument supports one or multiple targets expressed as one of the following, or a comma separated list of the following:

* `--target 127.0.0.1`, `--target www.google.com`, ... single target.
* `--target 127.0.0.1:22` single target with port.
* `--target 127.0.0.1, 192.168.1.1:80` comma separated list of targets.
* `--target @targets.txt` load a list of targets from a file.
* `--target 192.168.1.1-10`, `--target 192.168.1.1-10:22` IP range (with or without port).
* `--target 192.168.1.0/24`, `--target 192.168.1.0/24:22` CIDR (with or without port).
* `--target 10.0.0.1, 172.0.0.1:2222, @other-targets.txt, 192.168.1.1-10` any comma separated combination of them.

## Providing Credentials

The `--username`/`--payloads` and `--password`/`--key` arguments all support the same logic depending on the value passed to them:

* If the value provided is an existing file name, it'll be loaded as a wordlist.
* If the value provided is in the form of `@/some/path/*.txt` it'll be used as a [glob expression](https://docs.rs/glob/latest/glob/) to iterate matching files.
* If the value provided is in the form of `#<NUMBER>-<NUMBER>:<OPTIONAL CHARSET>`, it'll be used to generate all possible permutations of the given charset (or the default one if not provided) and of the given length. For instance: `#1-3` will generate all permutations from 1 to 3 characters using the default ASCII printable charset, while `#4-5:0123456789` will generate all permutations of digits of 4 and 5 characters.
* If the value provided is in the form of `[<NUMBER>-<NUMBER>]`, it'll be used as an integer range.
* If the value provided is in the form of `[<NUMBER>, <NUMBER>, <NUMBER>]`, it'll be used as comma separated list of integers.
* Anything else will be considered as a constant string.

For instance:

* `legba <plugin name> --username admin --password data/passwords.txt` will always use `admin` as username while loading the passwords from a wordlist.
* `legba <plugin name> --username data/users.txt --password data/passwords.txt` will load both from wordlists and use all combinations.
* `legba <plugin name> --username admin` will always use `admin` as username and attempt all permutations of the default printable ASCII charset between 4 and 8 characters (this is the default behaviour when a value is not passed).
* `legba <plugin name> --username data/users.txt --password '@/some/path/*.key'` will load users from a wordlist while testing all key files inside `/some/path`.
* `legba <plugin name> --username data/users.txt --password '#4-5:abcdef'` will load users from a wordlist while testing all permutations of the charaters `abcdef` 4 and 5 characters long.
* `legba <plugin name> --username data/users.txt --password '[10-999]'` will load users from a wordlist while testing all numbers from 10 to 999.
* `legba <plugin name> --username data/users.txt --password '[1, 2, 3, 4]'` will load users from a wordlist while testing the numbers 1, 2, 3 and 4.

### Iteration Logic

Iteration over these credentials can be controlled by the `-I, --iterate-by <ITERATE_BY>` argument. The `-I user` (the default) will iterate like this:

```
for user in usernames {
  for password in passwords {
     // rate limiting and delays happen here
     plugin.login(user, password)
  }
}
```

While `-I password` will invert the loop:

```
for password in passwords {
  for user in usernames {
     // rate limiting and delays happen here
     plugin.login(user, password)
  }
}
```

While both strategies will eventually produce the same results, using a different approach can be useful in [cases like this one](https://github.com/evilsocket/legba/issues/7), especially when using `--rate-limit` or `--wait` delays.

### Predefined Combinations

Another option is using the `-C, --combinations <FILENAME>` argument, this will load a predefined set of `username:password` combinations from the given filename.

## Main Options

| Option | Default | Description |
| ------ | ------- | ----------- |
| `-L, --list-plugins` | | List all available protocol plugins and exit. |
| `-R, --recipe <RECIPE>` | | Load a recipe from this YAML file. |
| `-T, --target <TARGET>` | | Single target host, url or IP address, IP range, CIDR, @filename or comma separated combination of them. |
| `-U, --payloads, --username <USERNAME>` | `#4-8` | Constant, filename, glob expression as `@/some/path/*.txt`, permutations as `#min-max:charset` / `#min-max` or range as `[min-max`] / `[n, n, n]`. |
| `-P, --key, --password <PASSWORD>` | `#4-8` | Constant, filename, glob expression as `@/some/path/*.txt`, permutations as `#min-max:charset` / `#min-max` or range as `[min-max`] / `[n, n, n]`. |
| `-C, --combinations <COMBINATIONS>` | | Load `username:password` combinations from this file. |
| `--separator <SEPARATOR>` | `:` | Separator if using the --combinations/-C argument. |
| `-I, --iterate-by <ITERATE_BY>` | `user` | Whether to iterate by user or by password [possible values: `user`, `password`] |
| `-S, --session <FILENAME>` | | Save and restore session information from this file. |
| `-O, --output <OUTPUT>` | | Save results to this file. |
| `--output-format <FORMAT>` | `text` | Output file format [possible values: text, jsonl] |
| `--timeout <TIMEOUT>` | `10000` | Connection timeout in milliseconds. |
| `--retries <RETRIES>` | `5` | Number of attempts if a request fails. |
| `--retry-time <TIME>` | `1000` | Delay in milliseconds to wait before a retry. |
| `--single-match` | |  Exit after the first positive match is found. | 
| `--ulimit <ULIMIT>` | `10000` | Value for ulimit (max open file descriptors). | 
| `--concurrency <VALUE>` | `10` |  Number of concurrent workers. |
| `--rate-limit <LIMIT>` | `0` | Limit the number of requests per second. |
| `-W, --wait <WAIT>` | `0` | Wait time in milliseconds per login attempt. |
| `--jitter-min <VALUE>` | `0` | Minimum number of milliseconds for random request jittering. |
| `--jitter-max <VALUE>` | `0` | Maximum number of milliseconds for random request jittering. |
| `-Q, --quiet` | | Do not report statistics. |
| `--generate-completions <GENERATE_COMPLETIONS>` | | Generate shell completions [possible values: bash, elvish, fish, powershell, zsh] |
| `-h, --help` | | Print help. |
| `-V, --version` | | Print version. |

For the full list of arguments including plugin specific ones run `legba --help`.