#!/usr/bin/env python3
"""Check the reviewed Python/Rust resolutions against OSV without installing them."""
import json
from pathlib import Path
import re
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parent


def main():
    packages = set()
    for line in (ROOT / 'requirements-plot.txt').read_text().splitlines():
        match = re.match(r'^([A-Za-z0-9_.-]+)==([^\s]+)', line)
        if match:
            packages.add(('PyPI', *match.groups()))
    if not packages:
        raise ValueError('no pinned Python dependencies')
    for lock in (ROOT / 'locks').glob('*.lock'):
        for package in tomllib.loads(lock.read_text())['package']:
            if 'source' in package:
                if (package['source'] != 'registry+https://github.com/rust-lang/crates.io-index' or
                        not re.fullmatch('[0-9a-f]{64}', package.get('checksum', ''))):
                    raise ValueError('reviewed lockfiles must use checksummed crates.io packages')
                packages.add(('crates.io', package['name'], package['version']))
    packages = sorted(packages)
    request = urllib.request.Request('https://api.osv.dev/v1/querybatch',
        data=json.dumps({'queries': [{'package': {'ecosystem': ecosystem, 'name': name}, 'version': version}
                                    for ecosystem, name, version in packages]}).encode(),
        headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)['results']
    if len(result) != len(packages):
        raise ValueError('incomplete vulnerability response')
    findings = [(package, item['vulns']) for package, item in zip(packages, result) if item.get('vulns')]
    if findings:
        for package, vulnerabilities in findings:
            print(f'{package}: {", ".join(v["id"] for v in vulnerabilities)}')
        raise SystemExit(1)
    print(f'No known OSV advisories for {len(packages)} pinned dependencies.')


if __name__ == '__main__':
    main()
