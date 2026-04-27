# libskills-protocol

**MCP (Model Context Protocol) server for the LibSkills ecosystem.**

Part of the [LibSkills](https://github.com/LibSkills) ecosystem — the Behavioral Knowledge Layer for open-source libraries.

## Status

✅ **Implemented** — MCP server with 4 tools, communicating via stdin/stdout JSON-RPC 2.0.

## MCP Tools

| Tool | Description |
|------|-------------|
| `get_skill` | Get complete skill (all knowledge files) by key |
| `search_skills` | Keyword search (name, tags, summary) |
| `find_skills` | Semantic search across skill file contents |
| `get_section` | Get a specific knowledge section (e.g., pitfalls.md) |

## Usage

### Build

```bash
cd libskills-protocol
cargo build --release
```

### AI IDE Configuration

Add to your AI IDE's MCP configuration (e.g., Claude Desktop, Cursor):

```json
{
  "mcpServers": {
    "libskills": {
      "command": "/path/to/libskills-protocol/target/release/libskills-mcp",
      "args": []
    }
  }
}
```

### Manual Test

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/libskills-mcp

echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/libskills-mcp

echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_skill","arguments":{"path":"cpp/gabime/spdlog"}}}' | ./target/release/libskills-mcp
```

### Integration Flow

```
User: "Write C++ logging code"
  → AI detects spdlog
  → AI calls MCP tool get_skill(cpp/gabime/spdlog)
  → AI reads pitfalls.md and safety.md
  → AI generates correct code with no hallucinations
```

## Architecture

```
AI IDE (Claude/Cursor)
  │
  │ MCP JSON-RPC via stdio
  ▼
libskills-mcp ─── reads from ─── libskills-registry/skills/
  │
  ├── get_skill        → full skill with all files
  ├── search_skills    → keyword match on index
  ├── find_skills      → content-based TF-IDF
  └── get_section      → single knowledge file
```

## Requirements

- The `libskills-registry` repo must be in a sibling directory, OR
- Skills must be cached at `~/.libskills/cache/`

## Related

- [libskills-cli](https://github.com/LibSkills/libskills-cli) — Full CLI with HTTP API server
- [HTTP API Reference](https://github.com/LibSkills/libskills-docs/blob/main/reference/api.md)

## License

Apache 2.0
