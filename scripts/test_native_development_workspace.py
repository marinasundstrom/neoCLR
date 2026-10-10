"""Compiler staging must stay short and retain diagnostics when a build fails."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    'development_bundle', Path(__file__).with_name('prepare-native-development-bundle.py'))
bundle = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bundle)


class CompilerWorkspaceTests(unittest.TestCase):
    def test_nested_evidence_path_does_not_lengthen_checkout_and_failure_cleans_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = root / 'original'
            original.mkdir()
            (original / 'manifest.json').write_text(json.dumps({'files': {}}))
            output = root / ('nested evidence ' * 6) / 'output'
            revision = 'a' * 40
            build_source = []

            def run(command, **kwargs):
                if command[:2] == ['dotnet', 'build']:
                    source = Path(command[2]).parents[2]
                    build_source.append(source)
                    self.assertTrue(source.is_dir())
                    self.assertEqual(source.parent, root / 'target')
                    self.assertNotIn(output, source.parents)
                    # Replay the failing Windows generator's actual path shape.
                    launch = (Path('D:/a/neoCLR/neoCLR/target') / source.name /
                              'src/Raven.CodeAnalysis/obj/generator-artifacts/BoundNodeGenerator/net10.0' /
                              'adb7af9d-9a25-4493-a7b6-ad12bf94dcc5/bin/BoundNodeGenerator/debug/BoundNodeGenerator.exe')
                    self.assertLess(len(str(launch)), 260)
                    return subprocess.CompletedProcess(command, 1, 'generator failed', '')
                return subprocess.CompletedProcess(command, 0, '', '')

            args = ['prepare', '--bundle', str(original), '--output', str(output),
                    '--build-compiler', '--compiler-revision', revision]
            with patch.object(bundle, 'ROOT', root), patch('sys.argv', args), \
                    patch.object(bundle.subprocess, 'check_output', return_value=revision), \
                    patch.object(bundle.subprocess, 'run', side_effect=run):
                with self.assertRaisesRegex(RuntimeError, 'generator failed'):
                    bundle.main()
            self.assertEqual(len(build_source), 1)
            self.assertFalse(build_source[0].exists())
            report = json.loads((output / 'preparation.json').read_text())
            self.assertEqual(report['commands'][-1]['exitCode'], 1)
            self.assertEqual(report['commands'][-1]['stdout'], 'generator failed')


if __name__ == '__main__':
    unittest.main()
