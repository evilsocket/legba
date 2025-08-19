# Model Context Protocol (MCP)

Legba's Model Context Protocol integration allows any LLMs to use any of its plugins in order to perform automated tasks.

To start Legba MCP server (an high level of concurrency is recommended in order to allow the AI to spawn multiple plugins concurrently):

```sh
sudo legba --mcp 127.0.0.1:3001 --concurrency 256
```

## Claude

Once it's running, you can use it with Claude via the `supergateway` package (Claude still doesn't support SSE but only STDIO, so we'll need this for the time being). 

Edit your `claude_desktop_config.json` file and add:

```json
{
  "mcpServers": {
    "Legba": {
      "command": "npx",
      "args": [
        "-y",
        "supergateway",
        "--sse",
        "http://127.0.01:3001/sse"
      ]
    }
  }
}
```

You should now be able to [ask the AI to perform tasks with Legba for you](https://www.youtube.com/watch?v=PJv4Z4uSAtE).

## Cline

Edit your `cline_mcp_settings.json` file and add:

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