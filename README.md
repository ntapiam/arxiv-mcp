# arxiv-mcp

An [MCP](https://modelcontextprotocol.io) server that lets AI assistants search arXiv, inspect paper metadata, download full paper sources to a local cache for reading, and retrieve BibTeX citations.

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
Returns an array of papers with `id`, `title`, `authors`, `published`, `summary`, `url`, and `categories`.

### `get_paper_info`
```json
{ "paper_id": "1706.03762" }
```
Returns full metadata for the paper. Supports both new-style IDs (`2301.07041`) and old-style (`cs/0612060`).

### `get_paper_bibtex`
```json
{ "paper_id": "1706.03762" }
```
Returns a ready-to-use BibTeX entry string sourced directly from `arxiv.org/bibtex/<id>`.

### `download_sources`
```json
{ "paper_id": "1706.03762" }
```
Downloads the source tarball from arXiv, extracts it to the local cache directory, and returns the local `path` and a list of extracted `files`. Subsequent calls for the same paper return immediately from cache.

### `list_cached_papers`
```json
{}
```
Returns all papers in the local cache. Each entry includes `paper_id`, `path`, `cached_at` (ISO 8601), and `outdated` (true if older than 30 days). When outdated entries exist, a top-level `warning` is included.

### `purge_outdated_cache`
```json
{}
```
**Irreversible.** Deletes all cached papers older than 30 days. Returns a list of `purged` IDs and a `count`.

## Local Cache

Sources are extracted to the platform cache directory:

| Platform | Path |
|----------|------|
| Linux | `~/.cache/arxiv-mcp/<paper-id>/` |
| macOS | `~/Library/Caches/arxiv-mcp/<paper-id>/` |
| Windows | `%LOCALAPPDATA%\arxiv-mcp\<paper-id>\` |

Old-style arXiv IDs (e.g. `cs/0612060`) are stored with `/` replaced by `_`.

Files persist indefinitely. Use `purge_outdated_cache` to clean up entries older than 30 days.

## Installation

Requires Rust ([install via rustup](https://rustup.rs)).

```bash
cargo install --git https://github.com/ntapiam/arxiv-mcp
```

Or from a local clone:

```bash
git clone https://github.com/ntapiam/arxiv-mcp
cargo install --path arxiv-mcp
```

## MCP Configuration

Add to your MCP host config (e.g. Claude Desktop's `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "arxiv": {
      "command": "arxiv-mcp"
    }
  }
}
```

## Development

```bash
cargo build                  # compile
cargo run                    # run server (stdio)
cargo test                   # run tests
cargo clippy -- -D warnings  # lint
cargo fmt                    # format
```

The server communicates over **stdio** using the MCP protocol (newline-delimited JSON-RPC). To test interactively:

```bash
npx @modelcontextprotocol/inspector arxiv-mcp
```

## CI

Builds are run on tagged releases (`v*`) across Linux, macOS (ARM), and Windows via GitHub Actions.

## Notes

- arXiv's API is public and requires no API key. Requests are rate-limited to ~3/s.
- Paper sources are `.tar.gz` archives containing LaTeX. PDFs are saved as `paper.pdf` when no source is available.
- The arXiv API is queried at `export.arxiv.org/api/query` (Atom feed).
- BibTeX entries are sourced from `arxiv.org/bibtex/<id>`.
