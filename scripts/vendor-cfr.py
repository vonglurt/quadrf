#!/usr/bin/env python3
"""Extract the regulation text from the LII e-CFR mirror pages in
resources/regulatory/ into vendor/cfr47/ as plain text.

The text of the Code of Federal Regulations is a work of the United States
Government and is not subject to copyright (17 U.S.C. § 105). LII's page
furniture (navigation, notices) is stripped; only the section text and its
amendment history are kept. ecfr.gov is the authoritative current text.

Run from the repository root:  python3 -I scripts/vendor-cfr.py
"""
import html, pathlib, re, sys, datetime

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "resources" / "regulatory"
DST = ROOT / "vendor" / "cfr47"

def clean(raw: str) -> str:
    t = re.sub(r"<script.*?</script>", "", raw, flags=re.S)
    t = re.sub(r"<style.*?</style>", "", t, flags=re.S)
    t = re.sub(r"</(p|div|h\d|li|tr|br)>", "\n", t, flags=re.I)
    t = html.unescape(re.sub(r"<[^>]+>", " ", t))
    t = re.sub(r"[ \t ]+", " ", t)
    t = re.sub(r"\n\s*\n+", "\n", t)
    return t

def body(t: str) -> str:
    i = t.find("prev | next")
    j = t.find("CFR Toolbox")
    if i < 0 or j < 0 or j <= i:
        raise ValueError("page layout not recognised")
    return t[i + len("prev | next"):j].strip()

def main() -> int:
    DST.mkdir(parents=True, exist_ok=True)
    n = 0
    for p in sorted(SRC.glob("cfr-47-*.html")):
        sec = p.stem.replace("cfr-47-", "")
        fetched = datetime.date.fromtimestamp(p.stat().st_mtime).isoformat()
        try:
            text = body(clean(p.read_text(encoding="utf-8", errors="replace")))
        except ValueError as e:
            print(f"skip {p.name}: {e}", file=sys.stderr)
            continue
        out = DST / f"47-CFR-{sec}.txt"
        header = (f"47 CFR § {sec}\n"
                  f"Source: Legal Information Institute mirror of the e-CFR, "
                  f"https://www.law.cornell.edu/cfr/text/47/{sec} (retrieved into resources/regulatory/ on {fetched}, the saved page's modification date)\n"
                  f"Status: text of a United States Government work, public domain (17 U.S.C. § 105). "
                  f"Authoritative current text: https://www.ecfr.gov/current/title-47/section-{sec}\n"
                  f"{'-'*78}\n")
        out.write_text(header + text + "\n", encoding="utf-8")
        n += 1
        print(f"wrote {out.relative_to(ROOT)} ({len(text)} chars)")
    return 0 if n else 1

if __name__ == "__main__":
    sys.exit(main())
