#!/usr/bin/env python3
"""
estimate_tokens.py - estimate LLM token counts for files, no API call needed.

Usage:
    python estimate_tokens.py FILE [FILE2 ...]
    python estimate_tokens.py "C:\\KEEL\\_memories\\*.md"

By default uses fast, zero-dependency heuristics. If `tiktoken` is installed it
ALSO reports an accurate BPE count (GPT-4o o200k_base) as a close proxy for
Claude - same ballpark, usually within ~10-20% for prose, closer for code.

For an EXACT Claude count, use Anthropic's Messages `count_tokens` API.
The heuristic auto-adjusts: denser/technical text (high chars-per-word) gets a
smaller divisor because it tokenizes into more pieces.
"""
import sys, os, glob, re


def heuristics(text):
    chars = len(text)
    words = len(re.findall(r"\S+", text))
    lines = text.count("\n") + 1
    cpw = chars / words if words else 0
    # plain prose ~4.0 chars/token; markdown/paths ~3.6; code/JSON-heavy ~3.3
    divisor = 4.0 if cpw <= 6 else (3.6 if cpw <= 8 else 3.3)
    return {
        "chars": chars, "words": words, "lines": lines, "cpw": cpw,
        "est": chars / divisor, "divisor": divisor,
        "lo": chars / 4.0, "hi": chars / 3.3,
    }


def tiktoken_count(text):
    try:
        import tiktoken
    except ImportError:
        return None
    for name in ("o200k_base", "cl100k_base"):
        try:
            enc = tiktoken.get_encoding(name)
            return name, len(enc.encode(text, disallowed_special=()))
        except Exception:
            continue
    return None


def n(x):
    return f"{x:,.0f}"


def report(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            text = f.read()
    except OSError as e:
        print(f"  ! {path}: {e}")
        return 0
    h = heuristics(text)
    tk = tiktoken_count(text)
    size = os.path.getsize(path)
    print(f"\n{path}")
    print(f"  {size:,} bytes - {h['chars']:,} chars - {h['words']:,} words "
          f"- {h['lines']:,} lines - {h['cpw']:.1f} chars/word")
    print(f"  heuristic ~= {n(h['est'])} tokens  (range {n(h['lo'])}-{n(h['hi'])}, /{h['divisor']})")
    if tk:
        print(f"  tiktoken[{tk[0]}] = {n(tk[1])} tokens  (accurate BPE; close proxy for Claude)")
    else:
        print(f"  (pip install tiktoken  for an accurate BPE count)")
    return tk[1] if tk else h["est"]


def main(argv):
    if not argv:
        print(__doc__)
        return
    paths = []
    for a in argv:
        g = glob.glob(a)
        paths.extend(g if g else [a])
    total = 0
    for p in paths:
        total += report(p)
    if len(paths) > 1:
        print(f"\n=== TOTAL ~= {n(total)} tokens across {len(paths)} files ===")


if __name__ == "__main__":
    main(sys.argv[1:])
