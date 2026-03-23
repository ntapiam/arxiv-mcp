# arxiv-mcp

An [MCP](https://modelcontextprotocol.io) server that lets AI assistants search arXiv, inspect
paper metadata, download full paper sources to a local cache for reading, and retrieve BibTeX
citations.

---

## Quick Start

**1. Install the binary**

```bash
# Precompiled — no Rust required (requires cargo-binstall)
cargo binstall arxiv-mcp

# Or download directly from GitHub Releases:
# https://github.com/ntapiam/arxiv-mcp/releases
```

**2. Add to your MCP host** (see [MCP Configuration](#mcp-configuration) for all hosts)

```bash
# Claude Desktop — edit ~/Library/Application Support/Claude/claude_desktop_config.json
# OpenCode      — edit opencode.json in your project root
```

**3. Verify it works**

Ask your agent: *"List my cached arXiv papers."*
The `list_cached_papers` tool should respond immediately (even with an empty list).

---

## Tools

| Tool | Description |
|------|-------------|
| `search_papers` | Search arXiv by keyword/query, returns ranked results |
| `get_paper_info` | Fetch metadata for a specific paper by arXiv ID |
| `get_paper_bibtex` | Fetch the BibTeX citation entry for a paper |
| `download_sources` | Download and extract a paper's source files to local cache |
| `list_cached_papers` | List all papers in the local cache, flagging outdated entries |
| `purge_outdated_cache` | Delete all cached papers older than 30 days |

### `search_papers`
```json
{ "query": "attention transformer", "max_results": 10 }
```
Returns an array of papers with `id`, `title`, `authors`, `published`, `summary`, `url`, and
`categories`. Supports arXiv field prefixes: `ti:`, `au:`, `cat:`, `abs:`.

### `get_paper_info`
```json
{ "paper_id": "1706.03762" }
```
Returns full metadata for the paper. Supports both new-style IDs (`2301.07041`) and
old-style (`cs/0612060`).

### `get_paper_bibtex`
```json
{ "paper_id": "1706.03762" }
```
Returns a ready-to-use BibTeX entry string sourced directly from `arxiv.org/bibtex/<id>`.

### `download_sources`
```json
{ "paper_id": "1706.03762" }
```
Downloads the source tarball from arXiv, extracts it to the local cache directory, and
returns the local `path` and a list of extracted `files`. Subsequent calls for the same
paper return immediately from cache. The agent should navigate and read files directly
from the returned path — not request file contents through the tool.

### `list_cached_papers`
```json
{}
```
Returns all papers in the local cache. Each entry includes `paper_id`, `path`,
`cached_at` (ISO 8601), and `outdated` (true if older than 30 days). When outdated
entries exist, a top-level `warning` is included.

### `purge_outdated_cache`
```json
{}
```
**Irreversible.** Deletes all cached papers older than 30 days. Returns a list of
`purged` IDs and a `count`.

---

## Local Cache

Sources are extracted to the platform cache directory:

| Platform | Path |
|----------|------|
| Linux | `~/.cache/arxiv-mcp/<paper-id>/` |
| macOS | `~/Library/Caches/arxiv-mcp/<paper-id>/` |
| Windows | `%LOCALAPPDATA%\arxiv-mcp\<paper-id>\` |

Old-style arXiv IDs (e.g. `cs/0612060`) are stored with `/` replaced by `_`.
Files persist indefinitely. Use `purge_outdated_cache` to clean up entries older than 30 days.

---

## Installation

**Precompiled binary** (recommended, no Rust required):

```bash
# requires cargo-binstall (cargo install cargo-binstall)
cargo binstall arxiv-mcp
```

Or download a binary directly from [GitHub Releases](https://github.com/ntapiam/arxiv-mcp/releases).

**From crates.io** (compiles from source, requires Rust):

```bash
cargo install arxiv-mcp
```

**From source** (latest unreleased):

```bash
cargo install --git https://github.com/ntapiam/arxiv-mcp
```

---

## MCP Configuration

### Claude Desktop

Edit `claude_desktop_config.json`:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |

```json
{
  "mcpServers": {
    "arxiv": {
      "command": "arxiv-mcp"
    }
  }
}
```

Restart Claude Desktop after saving. The `arxiv` server will appear in the tools list.

### OpenCode

Add to `opencode.json` in your project root (or `~/.config/opencode/opencode.json` globally):

```json
{
  "mcp": {
    "arxiv": {
      "type": "local",
      "command": ["arxiv-mcp"]
    }
  }
}
```

Then install the research skill so agents know how to use the server effectively
(see [Research Skill](#research-skill-opencode) below).

---

## Research Skill (OpenCode)

This repo ships an [OpenCode agent skill](https://opencode.ai/docs/skills/) that teaches
agents a structured workflow for conducting arXiv research: query design, reading metadata,
downloading and navigating LaTeX sources, and collecting citations.

The skill is auto-discovered by OpenCode when you clone or work inside this repository.
It is located at:

```
.opencode/skills/arxiv-research/SKILL.md
```

**To install it globally** (available in all your projects):

```bash
mkdir -p ~/.config/opencode/skills/arxiv-research
cp .opencode/skills/arxiv-research/SKILL.md ~/.config/opencode/skills/arxiv-research/SKILL.md
```

**To use it**, ask your agent to load the skill, or it will load it automatically when the
task involves arXiv research:

```
Load the arxiv-research skill and find the top papers on score-based generative models.
```

Agents can also load it explicitly:

```
skill({ name: "arxiv-research" })
```

The skill covers:
- arXiv query syntax (`ti:`, `au:`, `cat:`, `abs:`, date ranges)
- Step-by-step research workflow (search → inspect → download → cite)
- How to navigate extracted LaTeX source directories
- Common arXiv category codes
- Cache hygiene with `list_cached_papers` and `purge_outdated_cache`

---

## Development

```bash
cargo build                  # compile
cargo run                    # run server (stdio)
cargo test                   # run tests
cargo clippy -- -D warnings  # lint
cargo fmt                    # format
```

The server communicates over **stdio** using the MCP protocol (newline-delimited JSON-RPC).
To test interactively:

```bash
npx @modelcontextprotocol/inspector arxiv-mcp
```

---

## CI

Builds are run on tagged releases (`v*`) across Linux, macOS (ARM), and Windows via GitHub
Actions.

---

## Notes

- arXiv's API is public and requires no API key. Requests are rate-limited to ~3/s.
- Paper sources are `.tar.gz` archives containing LaTeX. PDFs are saved as `paper.pdf`
  when no source is available.
- The arXiv API is queried at `export.arxiv.org/api/query` (Atom feed).
- BibTeX entries are sourced from `arxiv.org/bibtex/<id>`.
