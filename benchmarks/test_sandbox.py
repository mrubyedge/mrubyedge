from pathlib import Path
import unittest
from unittest.mock import patch

import sandbox
import run


class Isolation(unittest.TestCase):
    def test_offline_container_exposes_only_source_outputs_and_cargo_cache(self):
        with patch.dict('os.environ', {'GH_TOKEN': 'secret', 'GITHUB_OUTPUT': '/host/control'}):
            command = sandbox.container_command('test', Path('/outputs'), Path('/cargo-cache'), ['--offline'])
        self.assertEqual(command[command.index('--network') + 1], 'none')
        self.assertIn('--read-only', command)
        self.assertIn('--cap-drop=ALL', command)
        self.assertIn('--security-opt=no-new-privileges', command)
        self.assertNotIn('--privileged', command)
        self.assertNotIn('secret', str(command))
        self.assertNotIn('/host/control', str(command))
        mounts = [command[i + 1] for i, arg in enumerate(command) if arg == '--mount']
        self.assertEqual(len(mounts), 3)
        self.assertTrue(mounts[0].endswith('dst=/workspace,readonly'))
        self.assertIn('--pids-limit=256', command)
        self.assertIn('--memory=6g', command)

    def test_dependency_change_invalidates_reviewed_lock_identity(self):
        import copy
        options = run.parse_options(['--no-default-features', '--features=wasi'])
        manifest = {'dev-dependencies': {'mruby-compiler2-sys': '0.5.0'}, 'dependencies': {'fnv': '1.0.7'}}
        original = run.dependency_key(manifest, options)
        changed = copy.deepcopy(manifest)
        changed['dependencies']['fnv'] = '2'
        self.assertNotEqual(original, run.dependency_key(changed, options))
        options.features = ['mruby-regexp']
        self.assertNotEqual(original, run.dependency_key(manifest, options))


if __name__ == '__main__':
    unittest.main()
