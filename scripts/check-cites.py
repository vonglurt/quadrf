#!/usr/bin/env python3
"""Cross-reference linter for the quadrf documents.

Checks that every reference to a numbered statement, finding, unknown,
candidate, analysis table or specification resolves to something that is
defined:
  S-nnn-k   defined by a "- S-nnn-k." statement in specs/SPEC-nnn-*.md
  F.nn.k    defined by a "- F.nn.k " finding in investigations/Gnn-*.md
  U-nnn-k   defined by a "| U-nnn-k |" row in a Known-unknowns table
  Tnn       defined by a line starting "Tnn " printed by analysis/linkbudget.py
  SPEC-nnn  a file specs/SPEC-nnn-*.md
  resources/<path>   a file under resources/ (skipped when resources/ is absent)
Run from the repository root:  python3 -I scripts/check-cites.py
Exit status 1 if any reference is unresolved."""
import re, sys, pathlib, subprocess

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOC_DIRS = ("specs", "investigations", "lab", "docs", "vendor/summary")
TOP = ("index.md", "backlog.md", "README.md", "AGENTS.md")

def md_files():
    out = []
    for d in DOC_DIRS:
        out += sorted((ROOT / d).rglob("*.md"))
    out += [ROOT / f for f in TOP if (ROOT / f).exists()]
    return out

def defined():
    s, f, u = set(), set(), set()
    for p in (ROOT / "specs").glob("SPEC-*.md"):
        t = p.read_text(encoding="utf-8")
        s |= set(re.findall(r"^\s*-\s+(S-\d{3}-\d+)\.", t, flags=re.M))
        u |= set(re.findall(r"^\|\s*(U-\d{3}-\d+)\s*\|", t, flags=re.M))
    for p in (ROOT / "investigations").glob("G*.md"):
        t = p.read_text(encoding="utf-8")
        f |= set(re.findall(r"^\s*-\s+(F\.\d{2}\.\d+)\s", t, flags=re.M))
    tables = set()
    try:
        out = subprocess.run([sys.executable, "-I", str(ROOT / "analysis" / "linkbudget.py")], capture_output=True, text=True, check=True).stdout
        tables = set(re.findall(r"^(T\d+)\s", out, flags=re.M))
    except Exception as e:  # noqa: BLE001
        print(f"warning: could not run analysis script: {e}")
    specs = {m.group(1) for p in (ROOT / "specs").glob("SPEC-*.md") for m in [re.match(r"(SPEC-\d{3})", p.name)] if m}
    return s, f, u, tables, specs

def expand_braces(s: str):
    """`a.{h,cpp}` -> [`a.h`, `a.cpp`]; one level is enough for our citations."""
    m = re.search(r"\{([^{}]*)\}", s)
    if not m:
        return [s]
    out = []
    for alt in m.group(1).split(","):
        out += expand_braces(s[: m.start()] + alt + s[m.end():])
    return out

def main() -> int:
    s_def, f_def, u_def, t_def, spec_def = defined()
    bad = 0
    res_root = ROOT / "resources"
    for p in md_files():
        rel = p.relative_to(ROOT).as_posix()
        t = p.read_text(encoding="utf-8")
        refs = []
        refs += [("S", x) for x in re.findall(r"\b(S-\d{3}-\d+)\b", t)]
        refs += [("F", x) for x in re.findall(r"\b(F\.\d{2}\.\d+)\b", t)]
        refs += [("U", x) for x in re.findall(r"\b(U-\d{3}-\d+)\b", t)]
        refs += [("SPEC", x) for x in re.findall(r"\b(SPEC-\d{3})\b", t)]
        # analysis tables: "analysis T17", "T17,", "(T18)" but not "T-1" test ids or "T1 model" ... accept all Tnn tokens
        refs += [("T", x) for x in re.findall(r"(?<![\w-])(T\d{1,2})(?![\w-])", t)]
        for kind, x in sorted(set(refs)):
            ok = {"S": x in s_def, "F": x in f_def, "U": x in u_def, "SPEC": x in spec_def, "T": x in t_def}[kind]
            if not ok and kind == "S" and x.startswith("S-nnn"):
                ok = True
            if not ok:
                bad += 1
                print(f"unresolved {kind}: {rel}: {x}")
        if res_root.exists():
            for raw in set(re.findall(r"resources/([A-Za-z0-9_./{},-]+)", t)):
                for path in expand_braces(raw.rstrip(".,;:")):
                    if path.endswith("/") or "*" in path or "{" in path:
                        continue
                    if not (res_root / path).exists():
                        bad += 1
                        print(f"missing resource: {rel}: resources/{path}")
    print(f"{len(md_files())} files checked; defined: {len(s_def)} S, {len(f_def)} F, {len(u_def)} U, {len(t_def)} tables, {len(spec_def)} specs; {bad} unresolved")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
