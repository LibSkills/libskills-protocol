use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{json, Value};

fn main() {
    let registry = find_registry();
    let state = AppState { registry };

    let stdin = io::stdin().lock();
    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = handle_message(&line, &state);
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{}", response).ok();
        stdout.flush().ok();
    }
}

struct AppState {
    registry: PathBuf,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JsonRpcMessage {
    jsonrpc: String,
    #[serde(default)]
    id: serde_json::Value,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

fn handle_message(line: &str, state: &AppState) -> String {
    let msg: JsonRpcMessage = match serde_json::from_str(line) {
        Ok(m) => m,
        Err(e) => {
            return json_rpc_error(
                serde_json::Value::Null,
                -32700,
                &format!("Parse error: {}", e),
            )
        }
    };

    let result = match msg.method.as_str() {
        "initialize" => handle_initialize(&msg),
        "tools/list" => handle_tools_list(&msg),
        "tools/call" => handle_tools_call(&msg, state),
        "notifications/initialized" => return String::new(),
        _ => json_rpc_error(msg.id, -32601, &format!("Method not found: {}", msg.method)),
    };

    result
}

fn handle_initialize(msg: &JsonRpcMessage) -> String {
    let result = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "libskills-mcp",
            "version": "0.1.0"
        }
    });
    json_rpc_result(msg.id.clone(), result)
}

fn handle_tools_list(msg: &JsonRpcMessage) -> String {
    let tools = json!({
        "tools": [
            {
                "name": "get_skill",
                "description": "Get the complete LibSkills skill for a library, including all knowledge files (overview, pitfalls, safety, lifecycle, threading, best-practices, performance, examples).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Skill key in format {language}/{author}/{name}, e.g. 'cpp/gabime/spdlog'"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "search_skills",
                "description": "Search the LibSkills registry by keyword. Matches against library name, tags, and summary.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search keyword"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum results to return (default: 10)"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "find_skills",
                "description": "Semantic search across skill file contents. Use natural language to find skills, e.g. 'fast C++ logging library' or 'async HTTP client for Python'.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query describing what you need"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum results to return (default: 10)"
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "get_section",
                "description": "Get a specific knowledge section from a skill, e.g. 'pitfalls.md' or 'safety.md'. Useful when the AI only needs a specific constraint or pitfall list.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Skill key, e.g. 'cpp/gabime/spdlog'"
                        },
                        "section": {
                            "type": "string",
                            "description": "Section filename, e.g. 'pitfalls.md', 'safety.md', 'overview.md'"
                        }
                    },
                    "required": ["path", "section"]
                }
            }
        ]
    });
    json_rpc_result(msg.id.clone(), tools)
}

fn handle_tools_call(msg: &JsonRpcMessage, state: &AppState) -> String {
    let params = match &msg.params {
        Some(p) => p,
        None => return json_rpc_error(msg.id.clone(), -32602, "Missing params"),
    };

    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

    let result = match name {
        "get_skill" => get_skill_tool(&arguments, state),
        "search_skills" => search_skills_tool(&arguments, state),
        "find_skills" => find_skills_tool(&arguments, state),
        "get_section" => get_section_tool(&arguments, state),
        _ => return json_rpc_error(msg.id.clone(), -32601, &format!("Unknown tool: {}", name)),
    };

    match result {
        Ok(content) => json_rpc_result(
            msg.id.clone(),
            json!({
                "content": [{"type": "text", "text": content}]
            }),
        ),
        Err(e) => json_rpc_result(
            msg.id.clone(),
            json!({
                "content": [{"type": "text", "text": format!("Error: {}", e)}],
                "isError": true
            }),
        ),
    }
}

// --- Tool implementations ---

fn get_skill_tool(args: &Value, state: &AppState) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or("Missing 'path' argument")?;
    let skill_dir = state.registry.join("skills").join(path);
    let skill_json = skill_dir.join("skill.json");

    if !skill_json.exists() {
        return Err(format!(
            "Skill '{}' not found. Available skills are indexed under the registry.",
            path
        ));
    }

    let meta: Value =
        serde_json::from_str(&fs::read_to_string(&skill_json).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    let mut output = String::new();
    output.push_str(&format!(
        "# {} — {}\n\n",
        meta["name"].as_str().unwrap_or("?"),
        meta["repo"].as_str().unwrap_or("?")
    ));
    output.push_str(&format!(
        "Language: {}\n",
        meta["language"].as_str().unwrap_or("?")
    ));
    output.push_str(&format!(
        "Tier: {}  Group: {}  Trust: {}  Risk: {}\n",
        meta["tier"].as_str().unwrap_or("?"),
        meta["group"].as_str().unwrap_or("?"),
        meta["trust_score"].as_i64().unwrap_or(0),
        meta["risk_level"].as_str().unwrap_or("?"),
    ));

    if let Some(tags) = meta["tags"].as_array() {
        let tag_list: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
        output.push_str(&format!("Tags: {}\n", tag_list.join(", ")));
    }

    // Read all knowledge files in priority order
    if let Some(files) = meta["files"].as_object() {
        for priority in &["P0", "P1", "P2", "P3"] {
            if let Some(list) = files.get(&**priority).and_then(|l| l.as_array()) {
                for f in list {
                    if let Some(filename) = f.as_str() {
                        let fp = skill_dir.join(filename);
                        if let Ok(content) = fs::read_to_string(&fp) {
                            output.push_str(&format!(
                                "\n--- FILE: {} ({} priority) ---\n\n{}\n",
                                filename, priority, content
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(output)
}

fn search_skills_tool(args: &Value, state: &AppState) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or("Missing 'query' argument")?
        .to_lowercase();
    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

    let index = load_index(state)?;
    let skills = index.get("skills").and_then(|s| s.as_array());

    let mut results: Vec<(String, u32)> = Vec::new();
    if let Some(skills) = skills {
        for skill in skills {
            let name = skill["name"]
                .as_str()
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            let tags: Vec<String> = skill["tags"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_lowercase()))
                        .collect()
                })
                .unwrap_or_default();
            let summary = skill["summary"].as_str().unwrap_or("").to_lowercase();
            let lang = skill["language"].as_str().unwrap_or("").to_lowercase();
            let key = skill["key"].as_str().unwrap_or("").to_lowercase();

            let mut score = 0u32;
            if name == query {
                score += 100;
            }
            if name.contains(&query) {
                score += 50;
            }
            if lang.contains(&query) {
                score += 60;
            }
            if tags.iter().any(|t| t.contains(&query)) {
                score += 30;
            }
            if summary.contains(&query) {
                score += 20;
            }
            if key.contains(&query) {
                score += 10;
            }

            if score > 0 {
                results.push((skill["key"].as_str().unwrap_or("?").to_string(), score));
            }
        }
    }

    results.sort_by_key(|b| std::cmp::Reverse(b.1));
    results.truncate(limit);

    if results.is_empty() {
        return Ok(format!("No skills found for '{}'", query));
    }

    let mut output = format!("{} results for '{}':\n\n", results.len(), query);
    for (i, (key, score)) in results.iter().enumerate() {
        output.push_str(&format!("{}. {} (relevance: {})\n", i + 1, key, score));
    }

    Ok(output)
}

fn find_skills_tool(args: &Value, state: &AppState) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or("Missing 'query' argument")?;
    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as usize;

    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        return Ok("No query tokens extracted.".into());
    }

    let skills_dir = state.registry.join("skills");
    let mut results: Vec<(String, f64)> = Vec::new();

    collect_and_score(&skills_dir, String::new(), &query_tokens, &mut results)?;

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    if results.is_empty() {
        return Ok(format!("No skills found for '{}'", query));
    }

    let max_score = results.first().map(|r| r.1).unwrap_or(1.0);
    let mut output = format!("{} results for '{}':\n\n", results.len(), query);
    for (i, (key, score)) in results.iter().enumerate() {
        let pct = if max_score > 0.0 {
            (score / max_score * 100.0) as i32
        } else {
            0
        };
        output.push_str(&format!("{}. {} (relevance: {}%)\n", i + 1, key, pct));
    }

    Ok(output)
}

fn get_section_tool(args: &Value, state: &AppState) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or("Missing 'path' argument")?;
    let section = args
        .get("section")
        .and_then(|s| s.as_str())
        .ok_or("Missing 'section' argument")?;

    let section = if section.ends_with(".md") {
        section.to_string()
    } else {
        format!("{}.md", section)
    };
    let file_path = state.registry.join("skills").join(path).join(&section);

    if !file_path.exists() {
        return Err(format!(
            "Section '{}' not found in skill '{}'",
            section, path
        ));
    }

    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    Ok(content)
}

// --- JSON-RPC helpers ---

fn json_rpc_result(id: Value, result: Value) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }))
    .unwrap_or_else(|_| "{}".into())
}

fn json_rpc_error(id: Value, code: i32, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    }))
    .unwrap_or_else(|_| "{}".into())
}

// --- Utility ---

fn load_index(state: &AppState) -> Result<Value, String> {
    let index_path = state.registry.join("index.json");
    let content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn collect_and_score(
    dir: &std::path::Path,
    prefix: String,
    query_tokens: &[String],
    results: &mut Vec<(String, f64)>,
) -> Result<(), String> {
    if dir.join("skill.json").exists() {
        let mut score = 0.0;
        // Read all .md files and score based on token matches
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let doc_tokens: std::collections::HashSet<String> =
                            tokenize(&content).into_iter().collect();
                        for token in query_tokens {
                            if doc_tokens.contains(token) {
                                score += 1.0;
                            }
                        }
                    }
                }
                if path.is_dir() && path.file_name().is_some_and(|n| n == "examples") {
                    // Count example files as bonus
                    if let Ok(examples) = fs::read_dir(&path) {
                        let count = examples
                            .flatten()
                            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                            .count();
                        score += count as f64 * 0.1;
                    }
                }
            }
        }
        if score > 0.0 {
            results.push((prefix, score));
        }
        return Ok(());
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                let name = entry.file_name().to_string_lossy().to_string();
                let new_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", prefix, name)
                };
                collect_and_score(&entry.path(), new_prefix, query_tokens, results)?;
            }
        }
    }
    Ok(())
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|s| !s.is_empty())
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}

fn find_registry() -> PathBuf {
    // Auto-detect registry
    let exe_path = std::env::current_exe().unwrap_or_default();
    let mut search = exe_path.clone();
    for _ in 0..6 {
        search = match search.parent() {
            Some(p) => p.to_path_buf(),
            None => break,
        };
        let candidate = search.join("libskills-registry");
        if candidate.exists() {
            return candidate;
        }
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    for ancestor in cwd.ancestors().take(6) {
        let candidate = ancestor.join("libskills-registry");
        if candidate.exists() {
            return candidate;
        }
    }

    // Read from local cache
    let home = std::env::var("HOME").unwrap_or_default();
    let cache_dir = PathBuf::from(&home).join(".libskills").join("cache");
    if cache_dir.exists() {
        return PathBuf::from(&home).join(".libskills");
    }

    PathBuf::from(".")
}
