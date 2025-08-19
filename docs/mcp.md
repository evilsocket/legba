# Model Context Protocol (MCP)

Legba's Model Context Protocol integration allows any LLMs to use any of its plugins in order to perform automated tasks.

To start Legba MCP server (an high level of concurrency is recommended in order to allow the AI to spawn multiple plugins concurrently):

## SSE Mode

```sh
legba --mcp 127.0.0.1:3001 --concurrency 256
```

## STDIO Mode

```sh
legba --mcp stdio --concurrency 256
```

## Claude

Edit your `claude_desktop_config.json` file and add (using STDIO mode in this example):

```json
{
  "mcpServers": {
    "Legba": {
      "command": "/path/to/legba",
      "args": [
        "--mcp",
        "stdio"
      ]
    }
  }
}
```

You should now be able to [ask the AI to perform tasks with Legba for you](https://www.youtube.com/watch?v=PJv4Z4uSAtE).

## Cline

Edit your `cline_mcp_settings.json` file and add (using SSE mode in this example):

```json
{
  "mcpServers": {
    "Legba": {
      "url": "http://localhost:3001/sse",
      "disabled": false,
      "autoApprove": [
        "show_session",
        "list_plugins",
        "start_session"
      ]
    }
  }
}
```