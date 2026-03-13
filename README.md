# arxiv-mcp

An [MCP](https://modelcontextprotocol.io) server that lets AI assistants search arXiv, inspect paper metadata, and download full paper sources to a local cache for reading.

## Tools

| Tool | Description |
|------|-------------|
| `search_papers` | Search arXiv by keyword/query, returns ranked results |
| `get_paper_info` | Fetch metadata for a specific paper by arXiv ID |
| `download_sources` | Download and extract a paper's source files to local cache |
| `list_cached_papers` | List all papers already downloaded to the local cache |

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

### `download_sources`
```json
{ "paper_id": "1706.03762" }
```
Downloads the source tarball from arXiv, extracts it to `~/.cache/arxiv-mcp/<paper-id>/`, and returns the local `path` and a list of extracted `files`. Subsequent calls for the same paper return immediately from cache.

### `list_cached_papers`
```json
{}
```
Returns all papers already in the local cache with their `paper_id` and local `path`.

## Local Cache

Sources are extracted to:
```
~/.cache/arxiv-mcp/
  1706.03762/       ← "Attention Is All You Need" LaTeX source
    ms.tex
    background.tex
    ...
  cs_0612060/       ← old-style IDs: "/" → "_"
    ...
```

Files persist indefinitely. Agents can read them directly from the returned path.

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

## Notes

- arXiv's API is public and requires no API key. Requests are rate-limited to ~3/s.
- Paper sources are `.tar.gz` archives containing LaTeX. PDFs are saved as `paper.pdf` when no source is available.
- The arXiv API is queried at `export.arxiv.org/api/query` (Atom feed).
