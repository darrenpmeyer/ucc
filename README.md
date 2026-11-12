# ucc

The **U**ni**c**ode **C**haracter emitter - emits a unicode character by
name, codepoint, or alias.

```sh
printf 'H%sO\n' "$(ucc 'SUBSCRIPT TWO')"   # H₂O
```

## Release downloads

Download packages from [GitHub Releases](https://github.com/darrenpmeyer/ucc/releases).
Linux x86-64 (`amd64`) and ARM64 (`aarch64`, Debian `arm64`) are supported.
ARM64 needs a 64-bit OS; 32-bit Raspberry Pi OS is not supported by these builds.
Packages target glibc 2.35 or newer, including Debian 12+ and Ubuntu 22.04+.
Private repository downloads require GitHub access.

Verify downloads with `sha256sum --check SHA256SUMS` (download all assets listed,
or select the lines for your downloaded files). On Debian/Ubuntu:

```sh
sudo apt install ./ucc_0.9.0-1_amd64.deb  # use arm64 on ARM64
sudo apt remove ucc                     # uninstall
```

Binary archives contain a filesystem tree under `usr/`, with the map path compiled
as `/usr/share/ucc`. Extract the matching archive, then install its contents:

```sh
tar -xzf ucc-0.9.0-linux-x86_64.tar.gz
sudo cp -a ucc-0.9.0-linux-x86_64/usr/. /usr/
ucc 'EM DASH'
```

To remove a manually installed archive, remove `/usr/bin/ucc`,
`/usr/share/ucc/printable.csv`, and `/usr/share/doc/ucc/`. Prefer the `.deb` or
Arch packaging when available so your package manager tracks these files.

The `ucc-<version>-aur.tar.gz` release asset contains separate `ucc` and `ucc-bin`
recipes with `.SRCINFO`. Extract it, enter one recipe directory, and run
`makepkg -si`. The source recipe compiles locally; the binary recipe downloads
this release's matching architecture. Both install `ucc` and cannot coexist.
These are prepared recipes, not automatic submissions to AUR. Public AUR use
requires public release downloads.

## License and character data

The project code is [MIT licensed](LICENSE). The generated Unicode character map
is covered by [Unicode License V3](LICENSE-UNICODE); see [NOTICE](NOTICE) and
[data provenance](DATA-PROVENANCE.md). Binary packages also include dependency
license notices under `/usr/share/doc/ucc/third-party/`.

## Build and install

Requires Rust and Cargo (edition 2024).

```sh
cargo build --release
target/release/ucc --charmap ./printable.csv 'EM DASH'
```

Use the Makefile to install both the binary and its character map:

```sh
make                         # release build
make check                   # formatting, Clippy, Rust and Python tests
sudo make install
sudo make uninstall
```

`make install` copies the existing release binary and character map without
building. Run `make` as your normal user before `sudo make install`.
For a user-local installation, run `make PREFIX="$HOME/.local"` followed by
`make install PREFIX="$HOME/.local"`.

`make install` defaults to `/usr/local/bin/ucc` and
`/usr/local/share/ucc/printable.csv`. `PREFIX` changes both paths; `BINDIR`
overrides the binary directory and `DATADIR` overrides the map directory
(default `$(PREFIX)/share/ucc`). The map directory is compiled into the binary,
so use the same settings for building and installing. `DESTDIR` supports staged
packaging without changing the compiled runtime path. For example,
`make PREFIX=/usr` followed by
`make install DESTDIR=/tmp/ucc-package PREFIX=/usr` stages the binary under
`/tmp/ucc-package/usr/bin` and the map under
`/tmp/ucc-package/usr/share/ucc`. Run `make help` for all targets, including `debug`,
`fmt`, `clean`, and `charmap` (explicitly regenerates the character map).

The default character map is read from the installed share directory at runtime,
so the installed binary works from any directory. With a direct Cargo build,
the default is `/usr/local/share/ucc/printable.csv`; set `UCC_DATADIR` at build
time to change the directory. For development without installing, pass
`--charmap ./printable.csv`. The map is not embedded in the executable. Output has no trailing newline. Errors and ambiguous
matches go to STDERR with exit status 1 (except match listings with `--pager`); no partial character sequence is emitted.

## Examples

Look up a character by name, codepoint, or an abbreviated name. `printf` adds a
newline here because `ucc` itself emits none:

```sh
printf '%s\n' "$(ucc 'EM DASH')"        # —
printf '%s\n' "$(ucc 'em dash')"        # — (case-insensitive)
printf '%s\n' "$(ucc U+2192)"           # →
printf '%s\n' "$(ucc 'SUBSCR ONE')"     # ₁ (matches SUBSCRIPT ONE)
printf 'H%sO\n' "$(ucc 'SUBSCRIPT TWO')"   # H₂O
```

Browse a broad search with a pager. Without `--pager`, an ambiguous search shows
up to 10 options on STDERR and exits non-zero:

```sh
ucc 'ARROW' --pager | less
ucc 'SUBSCR' --pager | less
```

Combine **two characters**: a base letter followed by a combining acute accent.
These are two Unicode codepoints that usually display as one accented letter:

```sh
printf '%s\n' "$(ucc 'LATIN SMALL LETTER E' \
  --combine 'COMBINING ACUTE ACCENT')"   # é
```

Combine **four characters**: two letters, each followed by its accent. This
produces four codepoints displayed as two accented letters, avoiding a tall
stack of marks that some terminal fonts handle poorly:

```sh
printf '%s\n' "$(ucc 'LATIN SMALL LETTER E' \
  --combine 'COMBINING ACUTE ACCENT' \
  --combine 'LATIN SMALL LETTER O' \
  --combine 'COMBINING DIAERESIS')"      # éö
```

`--combine` appends each character in order; it accepts ordinary letters as well
as combining marks. These examples use common Latin glyphs and accents, which
are widely supported in terminals, though appearance depends on the font.

Use a separate alias file without changing your default configuration:

```sh
printf 'dash,2014\naccent,U+0301\n' > /tmp/ucc-example-aliases.csv
printf '%s\n' "$(ucc --aliases /tmp/ucc-example-aliases.csv dash)"  # —
printf '%s\n' "$(ucc --aliases /tmp/ucc-example-aliases.csv U+0065 \
  --combine accent)"                                            # é
```

## Explanation

`ucc` takes a character "name" as an argument; this argument is case-insensitive. It then searches `printable.csv` to determine the code point of the character:

- exact match of name prints the unicode character for that codepoint to STDOUT
- a fuzzy match with only one distinct character also prints that char to STDOUT
- multiple possible matches prints the first 10 options to STDERR in the form `<char> (<name>)` and exits non-zero; additional matches are summarized as `.. and N more matches (use --pager) for more`

Users can define their own aliases in homedir config directory, using a simple CSV file in the form `<name>,<cp or char>` ; the user can supply a unicode codepoint or a literal character to use

`ucc` emits utf-8 encoded chars in all cases.

## Arguments

- `--pager` lists all matching characters as `<char> (<name>)`, one per line, to STDOUT and exits successfully. It uses the same lookup precedence and can be piped to `less`: `ucc LETTER --pager | less`. It cannot be combined with `--combine`.
- `--charmap` points to an alternative file with the same format as `printable.csv`
- `--combine <NAME>` appends a combining character after the symbol named; so `ucc EXAMPLE --combine 'EXAMPLE 2'` combines EXAMPLE and EXAMPLE 2 characters. `--combine` can be repeated. Unicode imposes no fixed maximum sequence length; rendering depends on the terminal and font.
- `--aliases` points to an alternative user alias file (default `~/.config/ucc/aliases.csv` or equivalent XDG config path)

## Lookup and aliases

Codepoints accept hexadecimal (`2014`), `U+2014`, or `0x2014`. Invalid Unicode
scalar values, including surrogates, are rejected. Direct codepoint lookup can
emit characters outside the printable map. Exact user aliases take precedence,
then codepoints, then exact map names and aliases, then fuzzy matches.
Fuzzy lookup is case-insensitive and accepts either a substring or word prefixes
in order, allowing intervening words. For example, `SUBSCR ONE` matches
`SUBSCRIPT ONE` and its alias `SUBSCRIPT DIGIT ONE`; both refer to `₁`.
`LAT CAP A` can match `LATIN CAPITAL LETTER A`. Word order matters; this does
not perform typo correction. The same rules apply to user aliases, `--combine`,
and `--pager`.
Matches referring to the same character count as one result.

Lookup batches the main argument and all `--combine` arguments. Exact user
aliases and direct codepoints are resolved without opening the character map.
Remaining names share one streaming CSV scan for exact matches; only names
without an exact match enter a second, shared scan for fuzzy matches. Repeated
queries reuse their results, while output follows the original argument order.
The full map is never retained in memory: the scanner keeps the current record
and matching results. Broad fuzzy searches can retain many results, especially
with `--pager`. Fuzzy lookup pays for two scans; exact lookup pays for one.
The map is fully validated whenever it is scanned, but unused maps are not
opened or validated. Fuzzy lookup requires a seekable character-map file.

The default alias path is `$XDG_CONFIG_HOME/ucc/aliases.csv`, falling back to
`$HOME/.config/ucc/aliases.csv`. A missing default file is ignored; an explicitly
specified missing file is an error. Alias files have no header:

```csv
dash,2014
smile,😀
letter,A
accent,U+0301
```

A single scalar value is treated as a literal character (including `A` or `1`);
use a prefix such as `U+0041` to disambiguate codepoints. Names are
case-insensitive; later duplicate aliases replace earlier ones. Quote CSV fields
containing commas or quotes. For example:

```sh
ucc letter --combine accent   # emits A followed by COMBINING ACUTE ACCENT
```

Combining arguments use the same lookup rules and are appended in argument order,
without normalization or rearrangement. They are not restricted to Unicode marks,
so aliases and other character sequences work as well.

Run checks with `cargo test` and `python3 -m unittest discover -s tests`.

## Extract printable character definitions

The printable char database can be regenerated from the [versioned Unicode definition XML archive](https://www.unicode.org/Public/18.0.0/ucdxml/). Extract the `ucd.all.flat.xml` to the repo directory and run:

```sh
python3 extract_printable.py ucd.all.flat.xml printable.csv
```

The standalone script uses only the Python standard library and streams the XML.
Both paths are optional and default to the paths shown above.
The CSV columns are `na`, `cp` (uppercase hexadecimal), `na1`, and `aliases`.
`aliases` is a JSON array combining nonempty `na1` and nested `name-alias` values,
with duplicates removed. Algorithmic names containing `#` expand to the codepoint.

Printable characters are categories L (letters), M (marks), N (numbers),
P (punctuation), and S (symbols), plus ASCII space. Controls, format characters,
surrogates, private-use characters, unassigned codepoints, and other separators
are excluded. The categories come from the XML, independent of Python's bundled
Unicode version. Combining marks are included; printability does not guarantee a
visible standalone glyph or font support.



## Maintainer release process

`.github/workflows/release.yml` builds and tests x86-64 and ARM64 natively in
Ubuntu 22.04 containers, packages `.deb` files and binary archives, tests Debian
and Ubuntu installations, and assembles source and AUR recipe archives plus
checksums. Branch/PR/manual runs upload Actions artifacts without publishing.
A `v<version>` tag must match `Cargo.toml`; after all checks pass, its run creates
a draft GitHub Release, uploads all assets, then publishes it.

For a release, update the manifest, lockfile, examples and release notes, commit,
and push the commit and its version tag (for example `v0.9.0`). GitHub Actions
needs repository write permission for the release job. Releases stay subject to
the repository's visibility. The workflow does not change visibility or submit
to AUR/APT repositories. Use the generated AUR recipes after assets are final;
do not replace published archives without updating their checksums.
