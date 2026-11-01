import copy
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

import report
import run


def fixture():
    expected = {'versions': [{'commit': 'a' * 40, 'dependency_key': 'b' * 64, 'lock_sha256': 'c' * 64}],
                'runner_sha256': 'd' * 64, 'workloads': {'integer': {'expected': 1, 'sha256': 'e' * 64}},
                'config': {'samples': 2, 'sample_ms': 100, 'warmup_ms': 300, 'features': ['wasi'],
                           'default_features': False, 'profile': 'release', 'modes': ['warm']}}
    data = {'schema_version': 1, 'status': 'complete', 'created_at': '2026-10-27T00:00:00+00:00',
            'runner_sha256': expected['runner_sha256'], 'workloads': copy.deepcopy(expected['workloads']),
            'config': copy.deepcopy(expected['config']), 'versions': [
                {'ref': 'a' * 40, 'commit': 'a' * 40, 'crate_version': '2.1.2', 'compiler': '=0.5.0',
                 'lock_sha256': 'c' * 64, 'results': {'integer/warm': run.summarize('0\t10\t1000\n1\t10\t2000', 2)}}]}
    return data, expected


def chunk(kind, payload):
    return struct.pack('>I', len(payload)) + kind + payload + struct.pack('>I', zlib.crc32(kind + payload))


def image(width=1, height=1):
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)) +
            chunk(b'IDAT', zlib.compress(b'\0\xff\xff\xff\xff')) + chunk(b'IEND', b''))


class ReportBoundary(unittest.TestCase):
    def test_metadata_cannot_override_trusted_revision_configuration_or_hashes(self):
        for mutate in [lambda d: d['versions'][0].update(commit='f' * 40),
                       lambda d: d['versions'][0].update(ref='[click](https://example.com)'),
                       lambda d: d['versions'][0].update(lock_sha256='f' * 64),
                       lambda d: d['config'].update(samples=100),
                       lambda d: d['workloads']['integer'].update(sha256='f' * 64),
                       lambda d: d.update(runner_sha256='f' * 64),
                       lambda d: d.update(status='running')]:
            data, expected = fixture()
            mutate(data)
            with self.assertRaises(ValueError):
                report.sanitize(data, expected)

    def test_measurements_must_match_bounded_samples(self):
        for mutate in [lambda m: m.update(median=1), lambda m: m.update(mean=float('nan')),
                       lambda m: m.update(stdev=float('inf')), lambda m: m.update(samples=[]),
                       lambda m: m['samples'][0].update(iterations=True),
                       lambda m: m['samples'][0].update(iterations=1_000_001),
                       lambda m: m['samples'][0].update(elapsed_ns=120_000_000_001)]:
            data, expected = fixture()
            mutate(data['versions'][0]['results']['integer/warm'])
            with self.assertRaises(ValueError):
                report.sanitize(data, expected)

    def test_duplicate_and_nonfinite_json_are_rejected(self):
        for data in ['{"a":1,"a":2}', '{"x":NaN}', '{"x":Infinity}']:
            with self.assertRaises(ValueError):
                report.load_json(data)

    def test_valid_png_and_malformed_images(self):
        report.png(image())
        for data in [b'<svg onload="alert(1)">', image(8193), image(8192, 8192), image()[:-1],
                     image() + b'junk', image().replace(b'IDAT', b'JDAT'),
                     image().replace(chunk(b'IEND', b''), chunk(b'tEXt', b'x'))]:
            with self.assertRaises((ValueError, zlib.error)):
                report.png(data)

    def test_report_markdown_is_regenerated_and_extra_files_are_rejected(self):
        data, expected = fixture()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            source = root / 'input'
            source.mkdir()
            (source / 'results.json').write_text(json.dumps(data))
            (source / 'results.md').write_text('[untrusted](https://example.com)\n<script>bad()</script>')
            for name in report.FILES[2:]:
                (source / name).write_bytes(image())
            report.export_report(source, expected, root / 'safe')
            self.assertNotIn('untrusted', (root / 'safe/results.md').read_text())
            self.assertIn('0.000', (root / 'safe/results.md').read_text())
            (source / 'unexpected.sh').touch()
            with self.assertRaises(ValueError):
                report.export_report(source, expected, root / 'rejected')

    def test_symlinks_ancestors_special_files_and_oversized_files_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            target = root / 'private.log'
            target.write_text('must not be uploaded')
            link = root / 'link.log'
            link.symlink_to(target)
            directory = root / 'alias'
            directory.symlink_to(root, target_is_directory=True)
            fifo = root / 'pipe.log'
            os.mkfifo(fifo)
            large = root / 'large.log'
            large.write_bytes(b'12345')
            for path in [link, directory / 'private.log', fifo, large]:
                with self.assertRaises(ValueError):
                    report.read_regular(path, 4)

    def test_log_export_does_not_follow_symlinks_or_copy_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            raw = root / 'raw'
            raw.mkdir()
            private = root / 'private'
            private.write_text('secret')
            (raw / 'leak.log').symlink_to(private)
            (raw / 'legitimate.log').write_text('safe')
            (raw / 'source').mkdir()
            (raw / 'source/source.log').write_text('source')
            report.collect_logs(raw, root / 'logs')
            files = list((root / 'logs').iterdir())
            self.assertEqual([p.read_text() for p in files], ['safe'])


if __name__ == '__main__':
    unittest.main()
