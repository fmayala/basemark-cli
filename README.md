# basemark-cli

A Rust CLI and MCP server for [Basemark](https://github.com/fmayala/basemark) — the minimal wiki for engineers.

Create, read, update, delete, search, and organize notes from your terminal. Pipe markdown in and out. Connect AI agents via MCP.

## Install

```bash
# From source
cargo install --path .

# Or via crates.io (when published)
cargo install basemark-cli
```

## Setup

```bash
# Point to your Basemark instance
basemark config set url https://basemark.wiki

# Set your API token (generate one at basemark.wiki/settings)
basemark config set token bm_abc123...

# Verify
basemark config show
```

Config is stored at `~/.basemark/config.toml`.

## Commands

### Documents

```bash
# Create a document
basemark create --title "My Note"

# Create with content from stdin (markdown)
echo "# Hello\n\nThis is **markdown**." | basemark create --title "My Note"

# Create in a specific collection
basemark create --title "Design Doc" --collection <collection-id>

# Read a document (outputs markdown)
basemark read <id>

# Read as raw Tiptap JSON
basemark read <id> --json

# Update title
basemark update <id> --title "New Title"

# Update content from stdin
cat updated.md | basemark update <id>

# Delete (requires --force)
basemark delete <id> --force

# List all documents
basemark list

# List documents in a collection
basemark list --collection <collection-id>
```

### Search

```bash
# Full-text search
basemark search "architecture"

# Output is JSON with id, title, and snippet
```

### Collections

```bash
# List collections
basemark collections list

# Create a collection
basemark collections create "Engineering"

# Delete a collection (requires --force)
basemark collections delete <id> --force
```

### Sharing

```bash
# Make a document public
basemark share <id> --public

# Make it private again
basemark share <id> --private

# Invite someone by email
basemark share <id> --invite user@example.com

# Get the share URL
basemark share <id> --url
```

### MCP Server

Start a local MCP server for AI agent integration (stdio transport):

```bash
basemark mcp
```

#### Claude Code integration

Add to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "basemark": {
      "command": "basemark",
      "args": ["mcp"]
    }
  }
}
```

#### Available MCP tools

| Tool | Description |
|------|-------------|
| `search_docs` | Full-text search across all documents |
| `read_doc` | Read a document's content as markdown |
| `create_doc` | Create a new document |
| `update_doc` | Update a document's title or content |
| `delete_doc` | Delete a document |
| `list_docs` | List documents, optionally filtered by collection |
| `list_collections` | List all collections |
| `create_collection` | Create a new collection |
| `share_doc` | Set document visibility or invite by email |

### Config

```bash
# Set a value
basemark config set url https://basemark.wiki
basemark config set token bm_abc123...

# Get a value
basemark config get url

# Show full config
basemark config show
```

## Output

All commands output JSON by default. Use `--pretty` for formatted output:

```bash
basemark list --pretty
```

## Piping

The CLI is designed for Unix pipes. Content goes in via stdin, comes out via stdout:

```bash
# Agent workflow: generate content and save it
echo "# Summary\n\nKey findings..." | basemark create --title "Research Summary"

# Read, transform, write back
basemark read <id> | sed 's/old/new/g' | basemark update <id>

# Search and process results
basemark search "todo" | jq '.[].id'
```

## Architecture

```
basemark-cli
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── client.rs        # HTTP API client (reqwest)
│   ├── config.rs        # ~/.basemark/config.toml
│   ├── convert.rs       # Markdown ↔ Tiptap JSON (pulldown-cmark)
│   ├── output.rs        # JSON/pretty output
│   ├── mcp.rs           # MCP server (rmcp, stdio)
│   └── commands/        # One file per command
```

The CLI is a thin wrapper around the [Basemark REST API](https://github.com/fmayala/basemark). All operations go over HTTP with bearer token auth. The MCP server exposes the same operations as native tool calls.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `reqwest` | HTTP client |
| `pulldown-cmark` | Markdown parsing |
| `rmcp` | MCP server (Model Context Protocol) |
| `serde` / `serde_json` | JSON serialization |
| `tokio` | Async runtime |
| `toml` | Config file parsing |

## License

MIT
