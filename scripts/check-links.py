#!/usr/bin/env python3
"""Check that every relative Markdown link in the tracked documents resolves
to an existing file or directory. Run from the repository root:
    python3 -I scripts/check-links.py
Exit status 1 if any link is broken."""
import re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
LINK = re.compile(r"\]\(([^)#\s]+)(#[^)]*)?\)")
SKIP = ("resources", ".git")

def main() -> int:
    bad = 0
    files = [p for p in ROOT.rglob("*.md") if not any(s in p.relative_to(ROOT).parts for s in SKIP)]
    for p in sorted(files):
        text = p.read_text(encoding="utf-8")
        for m in LINK.finditer(text):
            target = m.group(1)
            if "://" in target or target.startswith("mailto:"):
                continue
            t = (p.parent / target).resolve()
            if not t.exists():
                bad += 1
                print(f"broken: {p.relative_to(ROOT)} -> {target}")
    print(f"{len(files)} files checked, {bad} broken links")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
