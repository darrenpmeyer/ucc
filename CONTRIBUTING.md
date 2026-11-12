# Developing and contributing to ucc

For installation and command-line usage, see the [README](README.md).

## Development setup

Use Rust and Cargo with edition 2024 support, a C toolchain for linking, Make,
and Python 3. The extractor
and Python tests use only the Python standard library. Install the Rust
`rustfmt` and `clippy` components to run the full checks.

Run commands from the repository root. Normal builds use the checked-in
`printable.csv`; the Unicode XML is only needed to regenerate it.

```sh
make debug
target/debug/ucc --charmap ./printable.csv 'EM DASH'
```

For a release build without installing:

```sh
make
target/release/ucc --charmap ./printable.csv 'EM DASH'
```

You can also build directly with `cargo build --release --locked`.

## Checks

Before submitting changes, run:

```sh
make check                   # formatting, Clippy, Rust and Python tests
```

Individual commands are available for narrower checks:

```sh
make fmt                     # format Rust source
make lint                    # Clippy with warnings as errors
make test                    # Rust and Python tests
```

The test commands can also be run directly with `cargo test --locked` and
`python3 -m unittest discover -s tests`. Rust CLI tests are in `tests/cli.rs`,
Rust unit tests in `tests/unit/`, and extractor tests in
`tests/test_extract_printable.py`.

## Build, install, and packaging paths

Build as your normal user before installing. `make install` copies the existing
release binary and character map without building; see the [README](README.md)
for basic system-wide and user-local installation commands.

`make install` defaults to `/usr/local/bin/ucc` and
`/usr/local/share/ucc/printable.csv`. `PREFIX` changes both paths; `BINDIR`
overrides the binary directory and `DATADIR` overrides the map directory
(default `$(PREFIX)/share/ucc`). The map directory is compiled into the binary,
so use the same settings for building and installing. `DESTDIR` supports staged
packaging without changing the compiled runtime path. For example,
`make PREFIX=/usr` followed by
`make install DESTDIR=/tmp/ucc-package PREFIX=/usr` stages the binary under
`/tmp/ucc-package/usr/bin` and the map under `/tmp/ucc-package/usr/share/ucc`.
Run `make help` for all targets, including `debug`, `fmt`, `clean`, and
`charmap` (explicitly regenerates the character map).

The man page source is `man/ucc.1`. Keep it in sync with CLI behavior.
`make install` installs it under `$(PREFIX)/share/man/man1/ucc.1`; override
`MANDIR` to change the base man directory. `DOCDIR` overrides the documentation
directory. Both honor `DESTDIR`, as do the binary and character map. Debian
packages, binary archives, and both AUR recipes inherit the man page through
the shared install tree. Preview it with `man -l man/ucc.1`.

The default character map is read from the installed share directory at runtime,
so the installed binary works from any directory. With a direct Cargo build, the
default is `/usr/local/share/ucc/printable.csv`; set `UCC_DATADIR` at build time
to change the directory. For development without installing, pass
`--charmap ./printable.csv`. The map is not embedded in the executable.

## Lookup implementation

Lookup batches the main argument and all `--combine` arguments. Exact user
aliases and direct codepoints are resolved without opening the character map.
Remaining names share one streaming CSV scan for exact matches; only names
without an exact match enter a second, shared scan for fuzzy matches. Repeated
queries reuse their results, while output follows the original argument order.
The full map is never retained in memory: the scanner keeps the current record
and matching results. Broad fuzzy searches can retain many results, especially
with `--pager`. Fuzzy lookup pays for two scans; exact lookup pays for one. The
map is fully validated whenever it is scanned, but unused maps are not opened or
validated. Fuzzy lookup requires a seekable character-map file.

## Extract printable character definitions

Regenerate the character map from `ucd.all.flat.xml` in the repository root.
See [data provenance](DATA-PROVENANCE.md) for the exact input version and hashes
used for the checked-in map. Its original download URL was not recorded; a new
upstream download is not guaranteed to reproduce it. For data updates, use a
versioned Unicode XML archive and record its origin and hashes as described in
the provenance document.

```sh
python3 extract_printable.py ucd.all.flat.xml printable.csv
```

The standalone script uses only the Python standard library and streams the XML.
Both paths are optional and default to the paths shown above. The CSV columns
are `na`, `cp` (uppercase hexadecimal), `na1`, and `aliases`. `aliases` is a
JSON array combining nonempty `na1` and nested `name-alias` values, with
duplicates removed. Algorithmic names containing `#` expand to the codepoint.

Printable characters are categories L (letters), M (marks), N (numbers), P
(punctuation), and S (symbols), plus ASCII space. Controls, format characters,
surrogates, private-use characters, unassigned codepoints, and other separators
are excluded. The categories come from the XML, independent of Python's bundled
Unicode version. Combining marks are included; printability does not guarantee a
visible standalone glyph or font support.

## Maintainer release process

`.github/workflows/release.yml` builds and tests x86-64 and ARM64 natively in
Ubuntu 22.04 containers, packages `.deb` files and binary archives, tests Debian
and Ubuntu installations, and assembles source and AUR recipe archives plus
checksums. AUR package builds are tested on x86-64 only. Branch/PR/manual runs
upload Actions artifacts without publishing. A `v<version>` tag must match
`Cargo.toml`; after all checks pass, its run creates
a draft GitHub Release, uploads all assets, then publishes it.

For a release, update the manifest, lockfile, examples and release notes,
commit, and push the commit and its version tag (for example `v0.9.1`). GitHub
Actions needs repository write permission for the release job. The workflow
does not submit packages to AUR/APT repositories. Use the generated AUR recipes
after assets are final; do not replace published archives without updating their checksums.
