---
name: arxiv-research
description: Search arXiv papers, retrieve metadata and BibTeX citations, and download LaTeX source files to the local cache for deep reading. Use this skill whenever a task involves finding, reading, or citing academic papers from arXiv.
license: MIT
compatibility: opencode
metadata:
  domain: academic-research
  server: arxiv-mcp
---

## What this skill does

Guides you through using the `arxiv` MCP server to conduct end-to-end academic research:
searching for papers, reading metadata and abstracts, downloading full LaTeX source for
deep analysis, and collecting BibTeX citations.

---

## Prerequisites

The `arxiv` MCP server must be running and connected. Verify it is available by calling
`list_cached_papers` — if it responds (even with an empty list), the server is live.
If the tool is not found, direct the user to the installation instructions in `README.md`.

---

## Research workflow

### Step 1 — Search

Use `search_papers` to find candidate papers.

```json
{ "query": "attention is all you need transformer", "max_results": 10 }
```

**Query construction tips:**
- `ti:keyword` — search in title only
- `au:lastname` — search by author surname (e.g. `au:vaswani`)
- `cat:cs.LG` — filter to an arXiv category (see categories below)
- `abs:keyword` — search abstract text
- Combine with spaces (implicit AND): `ti:diffusion au:ho cat:cs.CV`
- Date filter: append `submittedDate:[20230101 TO 20240101]`
- Use specific terminology from the field; arXiv search is keyword-based, not semantic

The response includes `id`, `title`, `authors`, `published`, `summary`, `url`, `categories`.
Scan summaries to identify the most relevant papers before going deeper.

### Step 2 — Inspect metadata

For any paper that looks promising, call `get_paper_info` with the arXiv ID:

```json
{ "paper_id": "1706.03762" }
```

Both ID formats are supported:
- New style: `2301.07041`
- Old style: `cs/0612060`

Use the full abstract and category list to confirm relevance before downloading sources.

### Step 3 — Download source for deep reading

When you need to read the actual content of a paper (methods, proofs, tables, pseudocode),
download its LaTeX source:

```json
{ "paper_id": "1706.03762" }
```

The tool returns a local `path` and a list of extracted `files`. **Navigate and read those
files directly from the filesystem** — do not ask the tool to return file contents.

Typical source layout after extraction:
```
<path>/
  main.tex        ← main document (may have a different name)
  sections/       ← split chapters/sections
  figures/        ← figures (PDF, EPS, PNG)
  references.bib  ← bibliography
```

Start with the `.tex` file that contains `\documentclass` — this is the root document.
Follow `\input{}` and `\include{}` directives to find section files.

**Note:** If no LaTeX source is available, arXiv provides a `paper.pdf` instead.
Subsequent calls for the same paper return immediately from cache.

### Step 4 — Collect citations

When you need a formatted citation:

```json
{ "paper_id": "1706.03762" }
```

`get_paper_bibtex` returns a ready-to-use BibTeX entry sourced from `arxiv.org/bibtex/<id>`.
Collect entries for all relevant papers and present them together at the end of a research task.

### Step 5 — Cache hygiene

Check what is cached with `list_cached_papers` (no arguments required).
The response includes a `warning` field when any entry is older than 30 days.
Run `purge_outdated_cache` to remove stale entries when disk space is a concern —
this action is **irreversible**.

---

## Common arXiv categories

| Category | Field |
|----------|-------|
| `cs.AI` | Artificial Intelligence |
| `cs.LG` | Machine Learning |
| `cs.CV` | Computer Vision |
| `cs.CL` | Computation and Language (NLP) |
| `cs.RO` | Robotics |
| `cs.SE` | Software Engineering |
| `math.OC` | Optimization and Control |
| `stat.ML` | Statistics / Machine Learning |
| `quant-ph` | Quantum Physics |
| `cond-mat.*` | Condensed Matter Physics |

---

## Rate limiting

The arXiv API is public and requires no API key. The server enforces ~3 requests/second.
If you are running multiple searches in sequence, space them out slightly — the server
handles this automatically, but avoid tight loops of many calls.

---

## Example research session

**Task:** "Summarize the state of the art in diffusion models for image generation."

1. `search_papers` — `"diffusion models image generation"`, max 15
2. Scan summaries; pick 4–5 seminal or recent papers
3. `get_paper_info` on each to read full abstracts and confirm dates
4. `download_sources` on the 2–3 most relevant papers
5. Read `main.tex` (and any `sections/*.tex`) from each extracted path
6. `get_paper_bibtex` for all papers referenced in the summary
7. Return a structured summary + BibTeX block
