# AGENTS.md — arxiv-mcp

An MCP (Model Context Protocol) server in Rust that searches arXiv via its API and downloads
paper sources to a local cache for use in AI context.

---

## Build & Dev Commands

```bash
cargo build                     # compile
cargo run                       # run the MCP server (communicates over stdio)
cargo test                      # run all tests
cargo test <name_substring>     # run a single test, e.g.: cargo test test_list_detects_outdated
cargo clippy -- -D warnings     # lint; warnings are treated as errors in CI
cargo fmt                       # format code (must pass before committing)
cargo check --locked            # fast type-check only (used in CI)
cargo build --locked --release  # release build (used in CI)
```

Interactive testing via MCP inspector:
```bash
npx @modelcontextprotocol/inspector arxiv-mcp
```

CI (`.github/workflows/ci.yml`) runs: `cargo check`, `cargo clippy`, `cargo build --release`,
and `cargo test` — all with `--locked`.

---

## Version Control

This repo uses **Jujutsu (`jj`)**, not plain git. Use `jj-mcp-server` for all VCS operations.

- Commit frequently with descriptive messages: `jj commit -m "<message>"`
- All work goes on the **trunk** branch
- Prefer small, focused commits over large ones
- Do not use `git` commands directly

---

## Available MCP Servers (while coding)

| Server | Purpose |
|--------|---------|
| `lsp-mcp` | LSP integration — go-to-definition, hover, diagnostics, completions |
| `rust-mcp-server` | Cargo tooling — `cargo check`, `cargo test`, `cargo clippy`, `cargo add` |
| `jj-mcp-server` | Jujutsu VCS operations — all version control tasks |

---

## Architecture

```
stdio (JSON-RPC/MCP)
  → rmcp transport
    → ArxivServer (tool_router dispatch)
        search_papers       → ArxivClient::search      → parse_atom_feed (XML/Atom)
        get_paper_info      → ArxivClient::get_paper_info
        get_paper_bibtex    → ArxivClient::get_bibtex
        download_sources    → downloader::download_and_extract → ~/.cache/arxiv-mcp/<id>/
        list_cached_papers  → Cache::list
        purge_outdated_cache→ Cache::purge_outdated
```

**Module tree:**
```
crate (main.rs)
 ├── mod arxiv
 │    ├── mod api        → ArxivClient (HTTP + XML Atom feed parsing)
 │    └── mod types      → Paper struct (shared data type)
 ├── mod cache
 │    ├── (mod.rs)       → Cache (unit struct), CachedPaper, PurgeResult
 │    └── mod downloader → download_and_extract()
 └── mod server          → ArxivServer, param structs, MCP tool impls
```

**Key structs:**
- `ArxivServer` — MCP server; holds a `ToolRouter<Self>` and a shared `reqwest::Client`
- `ArxivClient` — thin wrapper over `reqwest::Client`; constructed per-request
- `Cache` — zero-field unit struct; all methods are `pub fn` statics (no instance state)
- `Paper` — serializable paper metadata (id, title, authors, published, summary, url, categories)
- `CachedPaper` — serializable cache entry (paper_id, path, cached_at, outdated)

---

## Code Style Guidelines

### Rust Edition
- **Edition 2024** (specified in `Cargo.toml`)

### Imports
- Standard library imports first, then third-party crates, then `crate::`/`super::` imports
- No blank-line grouping is enforced, but ordering follows the logical dependency direction
- Example:
  ```rust
  use quick_xml::events::Event;
  use quick_xml::Reader;
  use super::types::Paper;
  ```

### Naming Conventions
| Kind | Convention | Example |
|------|-----------|---------|
| Types / Structs / Enums | `PascalCase` | `ArxivServer`, `CachedPaper`, `PurgeResult` |
| Functions / Methods | `snake_case` | `search_papers`, `download_and_extract` |
| Constants | `SCREAMING_SNAKE_CASE` | `EXPIRY_DAYS`, `EXPIRY_DURATION` |
| Private helpers | module-private (no `pub`) | `extract_arxiv_id`, `parse_atom_feed` |
| MCP tool param structs | `PascalCase` + `Params` suffix | `SearchParams`, `PaperIdParams` |

### Formatting
- Run `cargo fmt` before every commit — CI enforces consistent formatting
- Clippy is run with `-D warnings`; fix all warnings before committing

### Error Handling
- Use `anyhow::Result<T>` for all internal/library-level fallible functions — no custom error types
- Create errors with `anyhow::anyhow!("message {}", val)` or `anyhow::bail!(...)`
- At MCP tool boundaries (`server.rs`), convert `anyhow::Error` to `String`:
  ```rust
  let papers = client.search(&params.query, max).await
      .map_err(|e| e.to_string())?;
  ```
- Use the `?` operator for early propagation throughout
- No `.unwrap()` in production code paths — use `?`, `.map_err(...)`, `.unwrap_or(...)`,
  `.unwrap_or_else(...)`, or `.unwrap_or_default()` instead
- On partial failures (e.g., mid-extraction), clean up manually before returning:
  ```rust
  let _ = std::fs::remove_dir_all(&dest);
  return Err(anyhow::anyhow!("extraction failed: {}", e));
  ```
- `.unwrap()` is acceptable inside `#[cfg(test)]` blocks

### Types and Traits
- Derive `serde::{Serialize, Deserialize}` on all structs that cross API or serialization boundaries
- Derive `schemars::JsonSchema` on all MCP tool parameter structs (alongside `Deserialize`)
- Use `#[serde(skip_serializing_if = "Option::is_none")]` to omit optional fields from JSON output
- Implement `ServerHandler` on `ArxivServer` via the `#[tool_handler]` proc-macro from `rmcp`
- Wire tool methods with `#[tool_router]` and `#[tool]` proc-macros — do not register tools manually

### Async
- All I/O is async with `tokio` (runtime: `#[tokio::main]`)
- HTTP requests use `reqwest`; share a single `reqwest::Client` instance across the server
- Rate-limit arXiv API calls to ~3 req/s

---

## Testing Patterns

- Tests live in `#[cfg(test)] mod tests { use super::*; ... }` inline in the source file
- Use `std::env::temp_dir()` for temporary directories; always clean up with `fs::remove_dir_all`
- Tests exercise internal logic directly (no HTTP mocking) — call module functions with controlled inputs
- The test runtime is synchronous (`#[test]`); use `#[tokio::test]` only if async logic must be tested
- No dev-dependencies are declared; use only `std` in tests unless a new dep is genuinely required

**Run a single test:**
```bash
cargo test test_list_detects_outdated
```

---

## Key Implementation Details

- arXiv API endpoint: `https://export.arxiv.org/api/query` (Atom/XML feed, no auth required)
- Paper sources: `.tar.gz` tarballs fetched from `https://arxiv.org/e-print/<paper-id>`
- Cache location: `~/.cache/arxiv-mcp/<paper-id>/` (resolved via `dirs::cache_dir()`)
- The `download_sources` tool must return the **local directory path**, not file contents — the
  agent navigates and reads files itself via the filesystem
- arXiv paper IDs: `2301.07041` (new format) or `cs/0612060` (old format)
- Cache expiry: entries older than `EXPIRY_DAYS` are flagged as outdated; `purge_outdated_cache`
  removes them. Timestamps are computed manually (no extra date crates).
- XML parsing uses `quick-xml` with a streaming event-based reader (not a DOM parser)
