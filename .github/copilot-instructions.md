# arxiv-mcp

An MCP (Model Context Protocol) server in Rust that searches arXiv via its API and downloads paper sources to a local cache for use in context.

## Build & Dev Commands

```bash
cargo build                     # compile
cargo run                       # run the server
cargo test                      # run all tests
cargo test <test_name>          # run a single test by name substring
cargo clippy -- -D warnings     # lint (treat warnings as errors)
cargo fmt                       # format code
```

## Version Control

This repo uses **Jujutsu (jj)**, not plain git. Use `jj` commands (via `jj-mcp-server`) for all VCS operations.

- Commit **frequently** with descriptive messages using `jj commit -m "<message>"`
- All work goes on the **trunk** branch
- Prefer small, focused commits over large ones

## Architecture

This is a Rust MCP server. The intended shape:

- **MCP transport layer** — speaks the MCP protocol (over stdio) to expose tools to an AI host
- **arXiv API client** — queries `export.arxiv.org/api/query` (Atom feed) for paper search/lookup
- **Source downloader** — fetches source tarballs from `arxiv.org/e-print/<id>`, extracts them into a permanent local cache directory, and returns the path to the agent
- **Local cache** — extracted paper sources live on disk indefinitely; tools return filesystem paths so the agent can read files directly

## Key Details

- Rust edition **2024** (see `Cargo.toml`)
- The arXiv API is public and requires no auth key; rate-limit to ~3 req/s
- Paper sources are `.tar.gz` archives; extract to `~/.cache/arxiv-mcp/<paper-id>/`
- The MCP tool for downloading should return the local directory path (not file contents) so the agent can navigate and read files itself
- arXiv paper IDs look like `2301.07041` or `cs/0612060`

## Available MCP Servers (while coding)

- **`lsp-mcp`** — LSP integration; use for go-to-definition, hover info, diagnostics, completions
- **`rust-mcp-server`** — Rust/Cargo tooling; use for `cargo check`, `cargo test`, `cargo clippy`, `cargo add`, etc.
- **`jj-mcp-server`** — Jujutsu VCS operations; use for all version control tasks
