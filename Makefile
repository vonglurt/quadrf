# quadrf — checks and tools (backlog D-06). POSIX sh; no bashisms.
PY := python3 -I
TMP := $(shell mktemp -d 2>/dev/null || echo /tmp/quadrf-make)

.PHONY: check lint links cites analysis sh-syntax rust parity deny audit vendor-cfr fetch import oracle corpus corpus-check clean

## check: everything that must pass before a commit
check: lint links cites analysis sh-syntax rust parity deny
	@echo "check: all passed"

lint:
	$(PY) scripts/lint-tags.py

links:
	$(PY) scripts/check-links.py

cites:
	$(PY) scripts/check-cites.py

analysis:
	python3 analysis/linkbudget.py > /dev/null

sh-syntax:
	sh -n scripts/fetch-resources.sh
	sh -n scripts/import-shared.sh
	sh -n scripts/build-oracle.sh

## rust: build the workspace with the locked dependency set (SPEC-008 S-008-13)
rust:
	@command -v cargo >/dev/null 2>&1 || { echo "cargo not found; skipping rust"; exit 0; }
	cargo build --locked --release --workspace
	cargo test --locked --release --workspace

## parity: the Rust analysis crate prints exactly what the Python script prints (backlog A-02)
parity: rust
	@command -v cargo >/dev/null 2>&1 || { echo "cargo not found; skipping parity"; exit 0; }
	@mkdir -p $(TMP)
	python3 analysis/linkbudget.py > $(TMP)/py.txt
	./target/release/qrf-analysis > $(TMP)/rs.txt
	@if diff $(TMP)/py.txt $(TMP)/rs.txt > $(TMP)/parity.diff; then echo "parity: identical ($$(wc -l < $(TMP)/py.txt) lines)"; else echo "parity: DIFFERENT"; head -40 $(TMP)/parity.diff; exit 1; fi

## deny / audit: supply-chain checks; skipped with a notice when the tools are not installed
deny:
	@if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; else echo "cargo-deny not installed (cargo install cargo-deny --locked); skipping"; fi

audit:
	@if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "cargo-audit not installed (cargo install cargo-audit --locked); skipping"; fi

vendor-cfr:
	$(PY) scripts/vendor-cfr.py

fetch:
	sh scripts/fetch-resources.sh

import:
	sh scripts/import-shared.sh

## oracle: build gr-lora_sdr (GPL-3.0; a separate process, never linked) into resources/oracle (backlog V-08)
oracle:
	sh scripts/build-oracle.sh

## corpus: the LoRa I/Q test corpus with the oracle's decode result per cell (backlog V-08); CORPUS_ARGS="--cells r08" restricts it
corpus:
	$(PY) scripts/make-corpus.py generate $(CORPUS_ARGS)

## corpus-check: every corpus file's sha256 against resources/corpus/qrf-lora-v1/MANIFEST.json
corpus-check:
	$(PY) scripts/make-corpus.py check

clean:
	rm -rf target
