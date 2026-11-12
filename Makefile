.DEFAULT_GOAL := build

CARGO ?= cargo
PYTHON ?= python3
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share/ucc
# The runtime path excludes DESTDIR, which is only a packaging staging root.
UCC_DATADIR := $(DATADIR)
export UCC_DATADIR
DOCDIR ?= $(PREFIX)/share/doc/ucc
DESTDIR ?=
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR

.PHONY: all build debug test lint fmt check install uninstall clean charmap help

all: build

build:
	$(CARGO) build --release --locked

debug:
	$(CARGO) build --locked

test:
	$(CARGO) test --locked
	$(PYTHON) -m unittest discover -s tests

lint:
	$(CARGO) clippy --locked --all-targets -- -D warnings

fmt:
	$(CARGO) fmt

check:
	$(CARGO) fmt -- --check
	$(MAKE) lint
	$(MAKE) test

install:
	install -d "$(DESTDIR)$(BINDIR)" "$(DESTDIR)$(DATADIR)" "$(DESTDIR)$(DOCDIR)"
	install -m 755 "$(CARGO_TARGET_DIR)/release/ucc" "$(DESTDIR)$(BINDIR)/ucc"
	install -m 644 printable.csv "$(DESTDIR)$(DATADIR)/printable.csv"
	install -m 644 README.md LICENSE LICENSE-UNICODE NOTICE DATA-PROVENANCE.md "$(DESTDIR)$(DOCDIR)/"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/ucc" "$(DESTDIR)$(DATADIR)/printable.csv"
	rm -f "$(DESTDIR)$(DOCDIR)/README.md" "$(DESTDIR)$(DOCDIR)/LICENSE" \
	  "$(DESTDIR)$(DOCDIR)/LICENSE-UNICODE" "$(DESTDIR)$(DOCDIR)/NOTICE" \
	  "$(DESTDIR)$(DOCDIR)/DATA-PROVENANCE.md"

clean:
	$(CARGO) clean

# Regeneration is explicit: regular builds use the checked-in character map.
charmap:
	$(PYTHON) extract_printable.py ucd.all.flat.xml printable.csv

help:
	@printf '%s\n' \
	  'make [build]  Build the release binary (default)' \
	  'make debug    Build the debug binary' \
	  'make test     Run Rust and Python tests' \
	  'make lint     Run Clippy with warnings as errors' \
	  'make fmt      Format Rust source' \
	  'make check    Check formatting, lint, and run tests' \
	  'make install  Install the prebuilt binary and map under PREFIX (default: /usr/local)' \
	  'make uninstall Remove the installed binary and character map' \
	  'make clean    Remove Cargo build artifacts' \
	  'make charmap  Regenerate printable.csv from the Unicode XML' \
	  'Overrides: CARGO, PYTHON, PREFIX, BINDIR, DATADIR, DESTDIR, CARGO_TARGET_DIR'
