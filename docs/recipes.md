# Recipes

Legba recipes are YAML files wrapping specific command line options and use cases, you can look at them as a templating engine for Legba. They are a convenient tool to alias otherwise complex arguments as a single mnemonic word. 

A "cookbook" with a few examples is [available here](https://github.com/evilsocket/legba-cookbook). For instance, this is the recipe to bruteforce a MS Exchange server via its auth.owa endpoint:

```yaml
description: Microsoft Exchange bruteforce via OWA endpoint.
author: evilsocket
plugin: http
args:
    target: "{$schema or https}://{$host}:{$port or 443}/owa/auth.owa"
    http-method: POST
    http-success-codes: 302
    http-success-string: set-cookie
    http-payload: destination={$schema or https}://{$host}:{$port or 443}/&flags=4&username={USERNAME}&password={PASSWORD}
```

This complex command line can now be executed simply via:

```bash
legba \
  -R cookbook/http/ms-exchange/owa.yml \
  -U users.txt \
  -P passwords.txt \
  "host=ms-server.local" 
```

### Variables

Recipes support a minimal template engine with the `{$variable_name or default_value}` syntax (or just `{$variable_name}` to make it mandatory for the user to provide). Each variable can be set via command line as:

```bash
legba \
  -R cookbook/http/ms-exchange/owa.yml \
  -U users.txt \
  -P passwords.txt \
  "host=ms-server.local&port=8443" 
```

### Resources

Another way of using recipes is including common dictionaries within their folder and referencing them in the YAML so that everything for that use case is self contained.

For instance, the [CVE-2023-46805 recipe](https://github.com/evilsocket/legba-cookbook/tree/main/http/vulnerabilities/CVE-2023-46805) contains a payloads.txt file that's being referenced like this:

```yaml
description: Tests one or multiple hosts for CVE-2023-46805.
author: https://twitter.com/assetnote/status/1747525904551842097
plugin: http.enum
args:
    target: "{$schema or https}://{$host}:{$port or 443}{$path or /}"
    payloads: "{$recipe.path}/payloads.txt"
    http-success-codes: "{$success_code or 200}"
    http-success-string: "Destination host"
    http-method: POST
```

Another example is the [LFI vulnerability testing recipe](https://github.com/evilsocket/legba-cookbook/tree/main/http/vulnerabilities/lfi):

```yaml
description: Performs common local file inclusion (LFI) vulnerabilities fuzzing.
author: evilsocket
plugin: http.enum
args:
    target: "{$schema or https}://{$host}:{$port or 443}{$path or /}"
    payloads: "{$recipe.path}/dictionary.txt"
    http-success-codes: "{$success_code or 200}"
    http-success-string: "root:"
```
