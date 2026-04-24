# libskills-protocol

**MCP and HTTP protocol definitions for the LibSkills ecosystem.**

Part of the [LibSkills](https://github.com/LibSkills) ecosystem — the Behavioral Knowledge Layer for open-source libraries.

## Status

📅 **Phase 10 on the roadmap.** Not yet implemented. Reserved for future use when multiple clients exist.

## MCP (Model Context Protocol)

When `libskills serve` is running, AI agents and IDEs can query skills via MCP:

```
Tool: get_skill
Input: { "path": "cpp/gabime/spdlog" }
Output: { skill metadata + file list }
```

## HTTP API (Future)

```
GET  /v1/skills                                       # List all skills
GET  /v1/skills/{language}/{author}/{name}             # Get full skill
GET  /v1/skills/{language}/{author}/{name}/{section}   # Get specific section
GET  /v1/search?q={keyword}                           # Search
GET  /v1/find?q={intent}                              # Semantic search
POST /v1/skills                                       # Submit Tier 2 skill
GET  /health                                          # Health check
```

## Constraints

Before this repository is active, the project must:
- Have proven that skills reduce AI errors (Phase 4)
- Have meaningful community adoption (Phase 5-6)
- Have multiple clients that would benefit from a server

See [libskills-docs/ROADMAP.md](https://github.com/LibSkills/libskills-docs/blob/main/ROADMAP.md) for the full phased plan.

## License

Apache 2.0
