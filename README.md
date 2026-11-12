# ucc

The **U**ni**c**ode **C**haracter emitter `ucc` emits a Unicode character by
name, codepoint, or alias.

```sh
printf 'H%sO\n' "$(ucc 'SUBSCRIPT TWO')"   # H₂O
```

## Installing

GitHub Releases provide downloads for Linux x86-64 and ARM64 machines:

- `.deb` releases for Debian 12+ / Ubuntu 22.04+
- Arch/AUR recipes `ucc` (build from sources) and `ucc-bin` (prebuilt binaries)
- `.tar.gz` archives of prebuilt binaries for other compatible Linux distributions

You can also build from sources with `make` and install globally with
`sudo make install`.

- [Downloading](#downloading)
- [Installing on Debian / Ubuntu](#installing-on-debian--ubuntu)
- [Installing with AUR recipes (Arch / Manjaro)](#installing-with-aur-recipes-arch--manjaro)
- [Installing binaries without a package manager](#installing-binaries-without-a-package-manager)
- [Installing from sources](#installing-from-sources)

### Downloading

Download packages from
[GitHub Releases](https://github.com/darrenpmeyer/ucc/releases). Linux x86-64
(`amd64`) and ARM64 (`aarch64`, Debian `arm64`) are supported. ARM64 needs a
64-bit OS; 32-bit Raspberry Pi OS is not supported by these builds. Packages
target glibc 2.35 or newer, including Debian 12+ and Ubuntu 22.04+.

Verify downloads with `sha256sum --check SHA256SUMS` (download all assets
listed, or select the lines for your downloaded files).

### Installing on Debian / Ubuntu

Download the `.deb` package for your architecture from
[GitHub Releases](https://github.com/darrenpmeyer/ucc/releases), then install
with:

```sh
sudo apt install ./ucc_0.9.1-1_amd64.deb  # use arm64 on ARM64
sudo apt remove ucc                       # uninstall
```

### Installing with AUR recipes (Arch / Manjaro)

Download `ucc-0.9.1-aur.tar.gz` from
[GitHub Releases](https://github.com/darrenpmeyer/ucc/releases). It contains two
recipes, each with a `PKGBUILD` and `.SRCINFO`:

- `ucc-bin` installs prebuilt binaries for `x86_64` or `aarch64`. `makepkg`
  automatically downloads the release archive for your architecture and verifies
  its SHA-256 checksum.
- `ucc` builds from source on either architecture. `makepkg -si` installs the
  required build dependencies, including Rust/Cargo and Python.

Install the Arch build tools, then extract the recipe bundle (replace `0.9.1`
with your downloaded version):

```sh
sudo pacman -S --needed base-devel
tar -xzf ucc-0.9.1-aur.tar.gz
cd ucc-0.9.1-aur/ucc-bin       # use ucc instead to build from source
```

Review the `PKGBUILD`, then run as your normal user:

```sh
makepkg -si
ucc 'EM DASH'
```

Choose one recipe: both provide the `ucc` command and cannot coexist. To
uninstall, run `sudo pacman -R ucc-bin` (or `sudo pacman -R ucc` for the source
package). These instructions use the recipes bundled with GitHub Releases;
releases do not automatically submit packages to the AUR.

### Installing binaries without a package manager

Binary archives contain a filesystem tree under `usr/`, with the map path
compiled as `/usr/share/ucc`. Extract the matching archive, then install its
contents:

```sh
tar -xzf ucc-0.9.1-linux-x86_64.tar.gz
sudo cp -a ucc-0.9.1-linux-x86_64/usr/. /usr/
ucc 'EM DASH'
```

To remove a manually installed archive, remove `/usr/bin/ucc`,
`/usr/share/ucc/printable.csv`, `/usr/share/man/man1/ucc.1`, and
`/usr/share/doc/ucc/`. Prefer the `.deb` or
Arch packaging when available so your package manager tracks these files.

### Installing from sources

Requires Rust and Cargo with edition 2024 support, Make, and a C toolchain
for linking. The character map is included in the repository; a normal build
does not require Python or the Unicode XML.
Run these commands from the repository root:

```sh
make                         # build as your normal user
sudo make install            # install the binary and character map
ucc 'EM DASH'
```

This installs under `/usr/local`. `make install` uses the existing release
binary, so always build before installing. To uninstall, run
`sudo make uninstall` from the repository root.

For a user-local installation (ensure `$HOME/.local/bin` is on your `PATH`):

```sh
make PREFIX="$HOME/.local"
make install PREFIX="$HOME/.local"
```

Use the same `PREFIX` when building, installing, and uninstalling: the character
map's installed location is compiled into the binary. See the
[contributor guide](CONTRIBUTING.md) for development setup, checks, and advanced
build and packaging options.

## Usage

Run `man ucc` for the installed command reference or `ucc --help` for a quick
option summary.

- [Arguments](#arguments)
- [Lookup and aliases](#lookup-and-aliases)
- [Examples and recipes](#examples-and-recipes)

### Arguments

```text
ucc [OPTIONS] NAME
```

Output is UTF-8 with no trailing newline. Errors, including ambiguous lookups,
go to STDERR with exit status 1; a failed lookup emits no partial character
sequence to STDOUT.

- `--pager` lists all matching characters as `<char> (<name>)`, one per line, to
  STDOUT and exits successfully when matches are found. It uses the same lookup
  precedence and can be piped to `less`: `ucc LETTER --pager | less`. It cannot
  be combined with `--combine`.
- `--charmap <FILE>` uses an alternative character map with the same CSV format
  as `printable.csv`.
- `--combine <NAME>` appends another character, including an ordinary letter or
  a combining mark. Repeat it to append more characters in order; rendering
  depends on the terminal and font.
- `--aliases <FILE>` uses an alternative user alias file (default
  `~/.config/ucc/aliases.csv` or equivalent XDG config path).
- `-h`, `--help` shows command-line help.
- `-V`, `--version` shows the version.

### Lookup and aliases

Codepoints accept hexadecimal (`2014`), `U+2014`, or `0x2014`. Invalid Unicode
scalar values, including surrogates, are rejected. Direct codepoint lookup can
emit characters outside the printable map. Exact user aliases take precedence,
then codepoints, then exact map names and aliases, then fuzzy matches. Fuzzy
lookup is case-insensitive and accepts either a substring or word prefixes in
order, allowing intervening words. For example, `SUBSCR ONE` matches
`SUBSCRIPT ONE` and its alias `SUBSCRIPT DIGIT ONE`; both refer to `₁`.
`LAT CAP A` can match `LATIN CAPITAL LETTER A`. Word order matters; this does
not perform typo correction. The same rules apply to user aliases, `--combine`,
and `--pager`. Matches referring to the same character count as one result.

The default alias path is `$XDG_CONFIG_HOME/ucc/aliases.csv`, falling back to
`$HOME/.config/ucc/aliases.csv`. A missing default file is ignored; an
explicitly specified missing file is an error. Alias files have no header:

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

Combining arguments use the same lookup rules and are appended in argument
order, without normalization or rearrangement. They are not restricted to
Unicode marks, so aliases and other character sequences work as well.

### Examples and recipes

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

## Licenses for code and character data

The project code is [MIT licensed](LICENSE). The generated Unicode character map
is covered by [Unicode License V3](LICENSE-UNICODE); see [NOTICE](NOTICE) and
[data provenance](DATA-PROVENANCE.md). Binary packages also include dependency
license notices under `/usr/share/doc/ucc/third-party/`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, tests, lookup
internals, character-map regeneration, and the maintainer release process.
