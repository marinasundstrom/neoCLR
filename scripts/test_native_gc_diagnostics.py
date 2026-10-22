"""Failed native qualification must expose diagnostics without its artifact download."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('native_gc', Path(__file__).with_name('validate-native-gc.py'))
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class DiagnosticsTests(unittest.TestCase):
    def test_failure_prints_bounded_diagnostics_and_keeps_full_logs(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'results'
            diagnostics = b'x' * 20000 + b'\ncompiler error C4996\n'
            commands = [SimpleNamespace(returncode=0, stdout=b'candidate\n', stderr=b''),
                        SimpleNamespace(returncode=1, stdout=diagnostics, stderr=b'failure details\n')]
            stderr = io.StringIO()
            with patch('sys.argv', ['validate-native-gc', '--output', str(output)]), \
                 patch.object(validator.platform, 'system', return_value='Windows'), \
                 patch.object(validator.platform, 'platform', return_value='Windows-test'), \
                 patch.object(validator.subprocess, 'run', side_effect=commands), \
                 contextlib.redirect_stderr(stderr), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(validator.main(), 1)
            self.assertIn('compiler error C4996', stderr.getvalue())
            self.assertIn('failure details', stderr.getvalue())
            self.assertLess(len(stderr.getvalue()), 17000)
            self.assertEqual((output / 'unicode-library.stdout.log').read_bytes(), diagnostics)
            self.assertFalse(json.loads((output / 'report.json').read_text())['passed'])


if __name__ == '__main__':
    unittest.main()
