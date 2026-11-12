#!/usr/bin/env python3
"""Package an existing native release build; requires Debian packaging tools."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def dependency_notices(destination):
    metadata = json.loads(run('cargo', 'metadata', '--locked', '--offline', '--format-version=1', cwd=ROOT))
    destination.mkdir()
    for package in metadata['packages']:
        if package['source'] is None:
            continue
        target = destination / f"{package['name']}-{package['version']}"
        target.mkdir()
        (target / 'METADATA.txt').write_text(
            f"{package['name']} {package['version']}\nLicense: {package['license']}\n"
            f"Source: {package.get('repository') or package['source']}\n")
        source = Path(package['manifest_path']).parent
        licenses = [p for p in source.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT'))]
        if not licenses:
            raise RuntimeError(f"No license notice found for {package['name']}")
        for notice in licenses:
            shutil.copy2(notice, target / notice.name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--arch', choices=('x86_64', 'aarch64'))
    group.add_argument('--notices-only', type=Path)
    args = parser.parse_args()
    if args.notices_only:
        dependency_notices(args.notices_only)
        return
    expected = {'x86_64': 'amd64', 'aarch64': 'arm64'}[args.arch]
    if run('dpkg', '--print-architecture') != expected:
        parser.error('Build packages natively on the requested architecture')
    version = json.loads(run('cargo', 'metadata', '--locked', '--offline', '--no-deps', '--format-version=1', cwd=ROOT))['packages'][0]['version']
    output = ROOT / 'dist'
    output.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='ucc-package-') as temporary:
        work = Path(temporary)
        stage = work / f'ucc-{version}-linux-{args.arch}'
        subprocess.run(['make', 'install', 'PREFIX=/usr', f'DESTDIR={stage}'], cwd=ROOT, check=True)
        docdir = stage / 'usr/share/doc/ucc'
        dependency_notices(docdir / 'third-party')
        # The archive is a filesystem tree rooted at /usr, shared with ucc-bin.
        with tarfile.open(output / f'{stage.name}.tar.gz', 'w:gz') as archive:
            archive.add(stage, arcname=stage.name)
        (work / 'debian').mkdir()
        (work / 'debian/control').write_text('Source: ucc\nSection: utils\nPriority: optional\nMaintainer: Darren P Meyer <6667118+darrenpmeyer@users.noreply.github.com>\n\nPackage: ucc\nArchitecture: any\nDescription: Emit Unicode characters by name, codepoint, or alias\n')
        dependencies = run('dpkg-shlibdeps', '-O', '-e' + str(stage / 'usr/bin/ucc'), cwd=work).removeprefix('shlibs:Depends=')
        shutil.copy2(ROOT / 'packaging/copyright', docdir / 'copyright')
        installed_size = (sum(p.stat().st_size for p in stage.rglob('*') if p.is_file()) + 1023) // 1024
        control = stage / 'DEBIAN'
        control.mkdir()
        (control / 'control').write_text(
            f'Package: ucc\nVersion: {version}-1\nArchitecture: {expected}\n'
            'Maintainer: Darren P Meyer <6667118+darrenpmeyer@users.noreply.github.com>\n'
            f'Depends: {dependencies}\nInstalled-Size: {installed_size}\nSection: utils\nPriority: optional\n'
            'Description: Emit Unicode characters by name, codepoint, or alias\n')
        subprocess.run(['dpkg-deb', '--root-owner-group', '--build', str(stage), str(output / f'ucc_{version}-1_{expected}.deb')], check=True)
    print('Packages written to', output)


if __name__ == '__main__':
    main()
