#!/usr/bin/env python3
"""Validate untrusted benchmark output before rendering, uploading or publishing."""
import argparse
import json
import itertools
import math
import os
from pathlib import Path
import re
import stat
import struct
import zlib

import run

REPORT_LIMIT = 2 * 1024 * 1024
IMAGE_LIMIT = 8 * 1024 * 1024
FILES = ('results.json', 'results.md', 'results-time.png', 'results-speedup.png')


def read_regular(path, limit):
    path = Path(path).absolute()
    # Reject symlink ancestors as well as the final file, before any host read.
    for parent in [*reversed(path.parents), path]:
        if parent.is_symlink():
            raise ValueError(f'symlink is forbidden: {parent.name}')
    if not stat.S_ISREG(path.stat().st_mode):
        raise ValueError(f'not a regular file: {path.name}')
    with path.open('rb') as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_size > limit:
            raise ValueError(f'not a bounded regular file: {path.name}')
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f'file too large: {path.name}')
    return data


def load_json(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key] = value
        return result
    def invalid(value):
        raise ValueError('non-finite JSON constant')
    try:
        return json.loads(data, object_pairs_hook=unique, parse_constant=invalid)
    except (RecursionError, UnicodeError) as error:
        raise ValueError('invalid JSON') from error


def text(value, limit=256):
    if not isinstance(value, str) or len(value) > limit or any(ord(c) < 32 and c != '\n' for c in value):
        raise ValueError('invalid report text')
    return value


def sanitize(data, expected):
    if not isinstance(data, dict) or data.get('schema_version') != 1 or data.get('status') != 'complete':
        raise ValueError('expected a complete schema-version-1 report')
    versions = data.get('versions')
    if not isinstance(versions, list) or len(versions) != len(expected['versions']):
        raise ValueError('unexpected versions')
    config = data.get('config')
    if not isinstance(config, dict) or any(config.get(k) != v for k, v in expected['config'].items()):
        raise ValueError('unexpected measurement configuration')
    if data.get('runner_sha256') != expected['runner_sha256'] or data.get('workloads') != expected['workloads']:
        raise ValueError('unexpected runner or workloads')
    result = {'schema_version': 1, 'status': 'complete', 'created_at': text(data.get('created_at')),
              'config': dict(expected['config']), 'runner_sha256': expected['runner_sha256'],
              'workloads': expected['workloads'], 'versions': [], 'environment': {}}
    environment = data.get('environment', {})
    if not isinstance(environment, dict):
        raise ValueError('invalid environment metadata')
    for key in ('platform', 'machine', 'processor', 'rustc', 'cargo', 'RUSTUP_TOOLCHAIN'):
        result['environment'][key] = text(environment.get(key, ''), 4096)
    keys = {f'{case}/{mode}' for case in expected['workloads'] for mode in expected['config']['modes']}
    for version, wanted in zip(versions, expected['versions']):
        if not isinstance(version, dict) or version.get('commit') != wanted['commit'] or version.get('ref') != wanted['commit']:
            raise ValueError('unexpected revision')
        lock_hash = version.get('lock_sha256', '')
        if not isinstance(lock_hash, str) or not re.fullmatch('[0-9a-f]{64}', lock_hash):
            raise ValueError('invalid lockfile hash')
        if wanted['lock_sha256'] and lock_hash != wanted['lock_sha256']:
            raise ValueError('reviewed lockfile changed')
        measurements = version.get('results')
        if not isinstance(measurements, dict) or measurements.keys() != keys:
            raise ValueError('unexpected measurements')
        clean = {'ref': wanted['commit'], 'commit': wanted['commit'], 'dirty': False,
                 'crate_version': text(version.get('crate_version')), 'compiler': text(version.get('compiler')),
                 'lock_sha256': lock_hash, 'dependency_key': wanted['dependency_key'],
                 'dependencies_reviewed': bool(wanted['lock_sha256']), 'results': {}}
        for key in sorted(keys):
            measurement = measurements[key]
            if not isinstance(measurement, dict) or measurement.get('unit') != 'ns/iteration':
                raise ValueError('invalid measurement')
            samples = measurement.get('samples')
            if not isinstance(samples, list) or len(samples) != expected['config']['samples']:
                raise ValueError('unexpected sample count')
            rows = []
            for index, sample in enumerate(samples):
                if not isinstance(sample, dict):
                    raise ValueError('invalid sample')
                iterations, elapsed = sample.get('iterations'), sample.get('elapsed_ns')
                if (type(iterations) is not int or not 1 <= iterations <= 1_000_000 or
                        type(elapsed) is not int or not 1 <= elapsed <= 120_000_000_000):
                    raise ValueError('out-of-range sample')
                rows.append(f'{index}\t{iterations}\t{elapsed}')
            calculated = run.summarize('\n'.join(rows), len(samples))
            for field in ('median', 'mean', 'min', 'stdev'):
                value = measurement.get(field)
                if (type(value) not in (int, float) or not math.isfinite(value) or
                        not math.isclose(value, calculated[field], rel_tol=1e-9, abs_tol=1e-9)):
                    raise ValueError('statistics do not match samples')
            clean['results'][key] = calculated
        result['versions'].append(clean)
    return result


def png(data):
    if not data.startswith(b'\x89PNG\r\n\x1a\n'):
        raise ValueError('invalid PNG signature')
    offset, compressed, header = 8, bytearray(), None
    ended = False
    while offset < len(data):
        if offset + 12 > len(data):
            raise ValueError('truncated PNG')
        size = struct.unpack('>I', data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        end = offset + 12 + size
        if end > len(data):
            raise ValueError('truncated PNG chunk')
        payload = data[offset + 8:offset + 8 + size]
        checksum = struct.unpack('>I', data[end - 4:end])[0]
        if zlib.crc32(kind + payload) != checksum:
            raise ValueError('invalid PNG checksum')
        if header is None:
            if kind != b'IHDR' or size != 13:
                raise ValueError('missing PNG header')
            width, height, depth, color, compression, filtering, interlace = struct.unpack('>IIBBBBB', payload)
            if (not 1 <= width <= 8192 or not 1 <= height <= 8192 or width * height > 16_000_000 or
                    depth != 8 or color not in (2, 6) or compression or filtering or interlace):
                raise ValueError('unsupported or oversized PNG')
            header = (width, height, 3 if color == 2 else 4)
        elif kind == b'IHDR':
            raise ValueError('duplicate PNG header')
        elif kind == b'IDAT':
            compressed.extend(payload)
        elif kind == b'IEND':
            if size or end != len(data):
                raise ValueError('invalid PNG end')
            ended = True
            break
        elif kind not in (b'tEXt', b'pHYs'):
            raise ValueError('unexpected PNG chunk')
        offset = end
    if not ended:
        raise ValueError('missing PNG end')
    width, height, channels = header
    expected = height * (1 + width * channels)
    decoder = zlib.decompressobj()
    raw = decoder.decompress(compressed, expected + 1)
    if len(raw) != expected or not decoder.eof or decoder.unused_data:
        raise ValueError('invalid PNG pixels')
    if any(raw[row * (1 + width * channels)] > 4 for row in range(height)):
        raise ValueError('invalid PNG filter')


def export_report(source, expected, destination, images=True):
    source, destination = Path(source), Path(destination)
    if source.is_symlink():
        raise ValueError('symlink report directory')
    names = {p.name for p in itertools.islice(source.iterdir(), len(FILES) + 1)}
    allowed = set(FILES) if images else {'results.json', 'results.md'}
    if not names <= allowed or 'results.json' not in names or (images and names != set(FILES)):
        raise ValueError('unexpected report files')
    report = sanitize(load_json(read_regular(source / 'results.json', REPORT_LIMIT)), expected)
    table = (run.table(report) + '\n').encode()
    # The producer's Markdown is never used as a publication input.
    if 'results.md' in names:
        read_regular(source / 'results.md', REPORT_LIMIT)
    output = {'results.json': (json.dumps(report, indent=2, allow_nan=False) + '\n').encode(), 'results.md': table}
    if images:
        for name in FILES[2:]:
            content = read_regular(source / name, IMAGE_LIMIT)
            png(content)
            output[name] = content
    destination.mkdir(parents=True, exist_ok=False)
    for name, content in output.items():
        (destination / name).write_bytes(content)
    return report


def collect_logs(source, destination):
    """Copy bounded diagnostics only; never let upload-artifact traverse raw output."""
    source, destination = Path(source).absolute(), Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    if source.is_symlink():
        return
    total = sum(p.stat().st_size for p in destination.iterdir() if p.is_file())
    count, visited = 0, 0
    pending = [source]
    while pending and visited < 10_000:
        directory = pending.pop()
        try:
            entries = os.scandir(directory)
        except OSError:
            continue
        with entries:
            for entry in entries:
                visited += 1
                if visited > 10_000 or count >= 100 or total >= 32 * 1024 * 1024:
                    return
                if entry.is_dir(follow_symlinks=False):
                    if entry.name not in ('source', 'build'):
                        pending.append(Path(entry.path))
                    continue
                name = entry.name
                if name != 'Cargo.lock' and not name.endswith(('.log', '.tsv')):
                    continue
                try:
                    data = read_regular(entry.path, min(4 * 1024 * 1024, 32 * 1024 * 1024 - total))
                except (OSError, ValueError):
                    continue
                # Never preserve user-controlled artifact path names.
                suffix = '.lock' if name == 'Cargo.lock' else Path(name).suffix
                (destination / f'{count:03d}{suffix}').write_bytes(data)
                total += len(data)
                count += 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('--expected', type=Path, required=True)
    parser.add_argument('--destination', type=Path, required=True)
    args = parser.parse_args()
    try:
        export_report(args.source, load_json(read_regular(args.expected, REPORT_LIMIT)), args.destination)
    except (OSError, ValueError, KeyError, TypeError, zlib.error) as error:
        parser.exit(1, f'Report rejected: {error}\n')


if __name__ == '__main__':
    main()
