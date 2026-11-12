#!/usr/bin/env python3
"""Generate checksummed AUR recipes from release assets; run in a clean checkout."""
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    version = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline', '--no-deps', '--format-version=1'], cwd=ROOT, text=True))['packages'][0]['version']
    dist = ROOT / 'dist'
    dist.mkdir(exist_ok=True)
    source = dist / f'ucc-{version}-source.tar.gz'
    subprocess.run(['git', 'archive', '--format=tar.gz', f'--prefix=ucc-{version}/', '-o', str(source), 'HEAD'], cwd=ROOT, check=True)
    replacements = {
        '@VERSION@': version,
        '@SOURCE_SHA256@': digest(source),
        '@X86_64_SHA256@': digest(dist / f'ucc-{version}-linux-x86_64.tar.gz'),
        '@AARCH64_SHA256@': digest(dist / f'ucc-{version}-linux-aarch64.tar.gz'),
    }
    with tempfile.TemporaryDirectory(prefix='ucc-aur-') as temporary:
        bundle = Path(temporary) / f'ucc-{version}-aur'
        for flavor, name in [('source', 'ucc'), ('bin', 'ucc-bin')]:
            text = (ROOT / f'packaging/PKGBUILD.{flavor}.in').read_text()
            for key, value in replacements.items():
                text = text.replace(key, value)
            target = bundle / name
            target.mkdir(parents=True)
            (target / 'PKGBUILD').write_text(text)
            # makepkg must run as an unprivileged user; use it on an Arch host.
            subprocess.run(['bash', '-n', str(target / 'PKGBUILD')], check=True)
        with tarfile.open(dist / f'ucc-{version}-aur.tar.gz', 'w:gz') as archive:
            archive.add(bundle, arcname=bundle.name)
    assets = sorted(p for p in dist.iterdir() if p.suffix in ('.gz', '.deb'))
    (dist / 'SHA256SUMS').write_text(''.join(f'{digest(p)}  {p.name}\n' for p in assets))


if __name__ == '__main__':
    main()
