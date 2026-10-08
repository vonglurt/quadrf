#!/usr/bin/env python3
"""Evidence-tag linter for the quadrf documents.

Every numbered statement (- F.nn.k …, - S-nnn-k …) in investigations/ and
specs/ must carry one or more of the tags [S] [D] [M] [C] (docs/00-process.md
§2). Lines under the headings "Inherited facts" and "Carry-forward" are lists
of references to statements made elsewhere and are not checked. The script
lists untagged statements and prints per-file tag counts; exit status 1 if any
statement is untagged.

Run from the repository root:  python3 -I scripts/lint-tags.py
"""
import re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
TAG = re.compile(r"`\[(S|D|M|C)\]`")
STMT = re.compile(r"^\s*-\s+(F\.\d+\.\d+|S-\d{3}-\d+)\b")
HEAD = re.compile(r"^#+\s+(.*)")
SKIP_HEADINGS = ("inherited facts", "carry-forward")

def main() -> int:
    bad = 0
    print(f"{'file':52}{'stmts':>6}{'[S]':>5}{'[D]':>5}{'[M]':>5}{'[C]':>5}{'untagged':>9}")
    for sub in ("investigations", "specs"):
        for p in sorted((ROOT / sub).glob("*.md")):
            n = {"S": 0, "D": 0, "M": 0, "C": 0}
            stmts = 0
            untagged = []
            skipping = False
            for i, ln in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
                h = HEAD.match(ln)
                if h:
                    skipping = any(k in h.group(1).lower() for k in SKIP_HEADINGS)
                    continue
                if skipping or not STMT.match(ln):
                    continue
                stmts += 1
                tags = TAG.findall(ln)
                if not tags:
                    untagged.append(i)
                for t in tags:
                    n[t] += 1
            rel = p.relative_to(ROOT).as_posix()
            print(f"{rel:52}{stmts:6}{n['S']:5}{n['D']:5}{n['M']:5}{n['C']:5}{len(untagged):9}")
            for i in untagged:
                bad += 1
                print(f"    untagged: {rel}:{i}")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
