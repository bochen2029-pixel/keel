# chunker — read any document fully, no matter how large

A self-contained, **token-aware** document chunker. It splits a file too big to
read in one shot into context-sized chunks at clean semantic boundaries, with
orientation metadata so a reader never loses its place. Reading all the chunks in
order = reading the whole document.

> **If you are an AI session that just hit a file bigger than your context window:**
> run the Quick Start below, then read `INDEX.md` followed by `chunk-001.md …
> chunk-NNN.md` in order. That's the whole document, guaranteed to fit.

---

## Quick Start (zero install — works on plain Python 3.7+)

```bash
# split a file into <file>.chunks\ (chunk-001.md … + INDEX.md + _manifest.json)
python chunker.py --budget 100000 "path\to\huge_file.md"

# just estimate + show the plan, write nothing
python chunker.py --plan "path\to\huge_file.md"

# print one chunk to stdout (e.g. for an agent to Read piece by piece)
python chunker.py --budget 100000 --stdout 3 "path\to\huge_file.md"
```

On Windows you can also use the launcher: `chunk.cmd --plan "file.md"`.

`--budget` is **tokens of content per chunk** — set it to roughly **half your
available context window** so each chunk fits with room to think. Default 100000.

---

## The "read it fully" workflow

1. `python chunker.py "bigfile.md"` → creates `bigfile.chunks\`.
2. Open `INDEX.md` — it lists every chunk, its token count, and its section.
3. Read `chunk-001.md`, `chunk-002.md`, … in order.
   - Each chunk opens with a `section:` breadcrumb (where you are in the doc).
   - From chunk 2 on, a short `recap:` quotes the tail of the previous chunk, so
     nothing is lost at the seam.
4. You have now read the entire document, in order, within budget.

---

## What makes it reliable

- **Real token counts.** Uses `tiktoken` (GPT-4o `o200k_base`, a close proxy for
  Claude) when installed; falls back to a density-aware char heuristic otherwise.
- **Stays under budget.** Reserves headroom for headers + recap, so a rendered
  chunk reliably fits the budget you set (verified across 50k/80k/180k).
- **Semantic splitting.** Splits at the highest boundary that fits —
  heading → page (`## Page N`) → paragraph → sentence → line → word. Never splits
  mid-word, never drops content (verified: zero data loss).
- **Real overlap.** Carries the previous chunk's trailing block(s) as a marked
  recap for continuity (not the broken/no-op overlap of older chunkers).

## Formats

| Works out of the box (stdlib) | Needs an optional library |
|---|---|
| `.md .txt .rst .org` · code (`.py .js .ts .rs .go .java .c .cpp .cs …`) · `.json` · `.jsonl` · `.html/.htm` | `.pdf` → PyMuPDF *or* pypdf *or* pdfplumber · `.docx` → python-docx |

`.jsonl` is split by whole records (never a torn line) — ideal for Claude Code
session transcripts.

## Optional power-ups

Everything degrades gracefully; install only what you need:

```bash
pip install tiktoken          # exact token counts (otherwise: good heuristic)
pip install pymupdf           # PDF reading (best); or:  pip install pypdf pdfplumber
pip install python-docx       # .docx reading
```

Run `python chunker.py` with no args to print full help.

---

## All options

```
python chunker.py FILE [FILE ...] [options]

  --budget N     content tokens per chunk (default 100000)
  --overlap N    recap tokens carried between chunks (default 600)
  --out DIR      output dir (default: <file>.chunks next to the source)
  --plan         estimate + show the plan; write nothing
  --stdout N     print chunk N (1-based) to stdout and exit
  --encoding ENC tiktoken encoding (default o200k_base)
```

Accepts globs and multiple files: `python chunker.py "docs\**\*.pdf"`.

## Files in this folder

- `chunker.py` — the chunker (this is the whole tool; self-contained).
- `estimate_tokens.py` — standalone "how many tokens is this file?" utility
  (the chunker has counting built in; this is the quick stand-alone check).
- `chunk.cmd` — Windows launcher (`chunk.cmd <args>` = `python chunker.py <args>`).
- `README.md` — this file.

## Provenance

Built 2026-06-13. Consolidates and supersedes three earlier chunkers
(`BC_Canon\_tools\universal_chunker.py`, `Claude-Titanic\intake\prepare.py`,
`BC_Canon\…\AEGIS_harness\scripts\ingest.py`) — keeping their good ideas (page
markers, comment headers, context-% reporting) and fixing their shared gaps:
no real token counting, no semantic splitting, broken/absent overlap.
Portable: no hardcoded paths, no required dependencies.
