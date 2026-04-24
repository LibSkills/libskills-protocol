# libskills-protocol

**MCP and HTTP protocol definitions for the LibSkills ecosystem.**

## MCP (Model Context Protocol)

When `libskills serve` is running, AI agents and IDEs can query skills via MCP:

```
Tool: get_skill
Input: { "path": "cpp/gabime/spdlog" }
Output: { skill metadata + file list }
```

## HTTP API (Future)

```
GET  /v1/skills                                     # List all skills
GET  /v1/skills/{language}/{author}/{name}           # Get full skill
GET  /v1/skills/{language}/{author}/{name}/{section} # Get specific section
GET  /v1/search?q={keyword}                         # Search
GET  /v1/find?q={intent}                            # Semantic search
POST /v1/skills                                     # Submit Tier 2 skill
GET  /health                                        # Health
```

## License

Apache 2.0
