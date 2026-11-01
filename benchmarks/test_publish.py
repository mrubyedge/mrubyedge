import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import publish
import report


class Publication(unittest.TestCase):
    def test_snapshot_drops_unexpected_files_retains_30_runs_and_has_no_parent(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            remote = root / 'remote.git'
            subprocess.run(['git', 'init', '--bare', '--quiet', str(remote)], check=True)
            real_git = publish.git
            def local_git(state, *args, **kwargs):
                if args[:3] == ('remote', 'add', 'origin'):
                    args = (*args[:3], str(remote))
                return real_git(state, *args, **kwargs)
            source = root / 'report'
            source.mkdir()
            for name in report.FILES:
                (source / name).write_text('fixture')
            with patch.object(publish, 'git', side_effect=local_git):
                revision, _ = publish.prepare(root / 'first', 'owner/repo', source, '1', '1')
                publish.push(root / 'first')
                seed = root / 'first'
                publish.git(seed, 'read-tree', revision)
                blob = publish.git(seed, 'hash-object', '-w', '--stdin', data=b'untrusted branch data')
                for number in range(2, 36):
                    publish.git(seed, 'update-index', '--add', '--cacheinfo', f'100644,{blob},runs/{number}-1/results.json')
                publish.git(seed, 'update-index', '--add', '--cacheinfo', f'100644,{blob},.gitattributes')
                tree = publish.git(seed, 'write-tree')
                malicious = publish.git(seed, 'commit-tree', tree, data=b'fixture\n')
                publish.git(seed, 'push', '--force', 'origin', f'{malicious}:{publish.BRANCH}')
                latest, destination = publish.prepare(root / 'second', 'owner/repo', source, '36', '1')
                publish.push(root / 'second')
                paths = publish.git(root / 'second', 'ls-tree', '-r', '--name-only', latest).splitlines()
                self.assertNotIn('.gitattributes', paths)
                self.assertNotIn('runs/1-1/results.json', paths)
                self.assertEqual(len({p.rsplit('/', 1)[0] for p in paths}), 30)
                self.assertEqual(destination, 'runs/36-1')
                self.assertEqual(publish.git(root / 'second', 'rev-list', '--parents', '-n', '1', latest), latest)
                self.assertFalse((root / 'second/runs').exists())

    def test_stale_publication_cannot_overwrite_another_push(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            remote = root / 'remote.git'
            subprocess.run(['git', 'init', '--bare', '--quiet', str(remote)], check=True)
            real_git = publish.git
            def local_git(state, *args, **kwargs):
                if args[:3] == ('remote', 'add', 'origin'):
                    args = (*args[:3], str(remote))
                return real_git(state, *args, **kwargs)
            source = root / 'report'
            source.mkdir()
            for name in report.FILES:
                (source / name).write_text('fixture')
            with patch.object(publish, 'git', side_effect=local_git):
                publish.prepare(root / 'a', 'owner/repo', source, '1', '1')
                publish.prepare(root / 'b', 'owner/repo', source, '2', '1')
                publish.push(root / 'a')
                with self.assertRaises(subprocess.CalledProcessError):
                    publish.push(root / 'b')


if __name__ == '__main__':
    unittest.main()
