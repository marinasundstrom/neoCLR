"""Failure reporting must survive a reduced host run or an accidentally empty filter."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('validate_host', Path(__file__).with_name('validate-host.py'))
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class HostEvidenceTests(unittest.TestCase):
    def run_validator(self, respond, error):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / 'evidence'
            with patch.object(sys, 'argv', ['validate-host.py', '--output', str(output)]), \
                 patch.object(validator.subprocess, 'check_output', return_value='a' * 40), \
                 patch.object(validator.platform, 'platform', return_value='test-host'), \
                 patch.object(validator.subprocess, 'run', side_effect=respond), \
                 contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(RuntimeError, error):
                    validator.main()
            return json.loads((output / 'report.json').read_text())

    def test_nonzero_command_retains_failure_evidence(self):
        report = self.run_validator(lambda cmd, **kw: subprocess.CompletedProcess(cmd, 7, 'failure\n'),
                                    'Host check failed')
        self.assertEqual(report['status'], 'failed')
        self.assertEqual(report['commands'][-1]['exit_code'], 7)

    def test_empty_unit_selection_is_not_a_pass(self):
        def respond(cmd, **kw):
            return subprocess.CompletedProcess(cmd, 0, 'running 0 tests\n' if '--lib' in cmd else 'running 1 test\n')
        report = self.run_validator(respond, 'selected no tests')
        self.assertEqual(report['status'], 'failed')
        self.assertIn('selected no tests', report['error'])


if __name__ == '__main__':
    unittest.main()
