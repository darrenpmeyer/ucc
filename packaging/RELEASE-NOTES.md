Emit Unicode characters by name, codepoint, or alias, with custom aliases,
fuzzy lookup, combined sequences, and pager-friendly match listings.

Downloads support Linux x86-64 and ARM64 (AArch64). ARM64 requires a 64-bit OS,
including on Raspberry Pi. GNU/Linux runtime baseline: glibc 2.35;
Debian 12+, Ubuntu 22.04+, and compatible distributions.

- Debian/Ubuntu: download the matching `amd64.deb` or `arm64.deb`, then run
  `sudo apt install ./ucc_0.9.0-1_<architecture>.deb`.
- Other Linux distributions: extract the matching binary archive and run
  `sudo cp -a ucc-0.9.0-linux-<architecture>/usr/. /usr/`.
- Arch: the AUR recipe archive contains `ucc` (source build) and `ucc-bin`
  (prebuilt), with `.SRCINFO` and checksums. Extract the archive, enter the
  desired recipe directory, and run `makepkg -si`. Recipes require accessible
  GitHub release downloads; a private repository cannot serve public AUR users.

Verify downloaded assets against `SHA256SUMS`. Binary packages include the
character map, MIT license, Unicode notice, data provenance, and dependency
license notices. This release does not publish packages to AUR or APT repositories.
