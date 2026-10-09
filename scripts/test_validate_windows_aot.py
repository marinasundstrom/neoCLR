import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('windows_aot', Path(__file__).with_name('validate-windows-aot.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ExecutionGateTests(unittest.TestCase):
    def test_missing_skipped_or_partial_execution_cannot_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'windows-execution.json'
            with self.assertRaises(FileNotFoundError):
                module.require_execution(path)
            for report in ({'passed': False, 'outcomes': [{}] * 32},
                           {'passed': True, 'outcomes': []},
                           {'passed': True, 'outcomes': [{}] * 31}):
                path.write_text(json.dumps(report))
                with self.assertRaises(ValueError):
                    module.require_execution(path)

    def test_completed_native_gate_is_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'windows-execution.json'
            report = {'passed': True, 'outcomes': [{}] * 32}
            path.write_text(json.dumps(report))
            self.assertEqual(module.require_execution(path), report)


if __name__ == '__main__':
    unittest.main()
