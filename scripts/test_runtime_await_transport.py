"""Qualification transport must preserve UTF-8 and exact line delimiters."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('runtime_await', Path(__file__).with_name('validate-runtime-await-sdk.py'))
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class TransportTests(unittest.TestCase):
    def test_subprocess_receives_bytes_without_text_mode(self):
        payload = 'http://localhost/greeting\nnormal\n'
        with patch.object(validator.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, b'OK\r\n', b'')) as run:
            result = validator.run_utf8(['guest'], input=payload, timeout=10)
        run.assert_called_once_with(['guest'], capture_output=True, input=payload.encode('utf-8'), timeout=10)
        self.assertEqual(result.stdout, 'OK\r\n')

    def test_real_child_preserves_unicode_and_mixed_delimiters(self):
        payload = 'Café 🌍\nnormal\r\n'
        result = validator.run_utf8([sys.executable, '-c',
            'import sys; data = sys.stdin.buffer.read(); sys.stdout.buffer.write(data); sys.stderr.buffer.write("ö".encode("utf-8"))'], input=payload)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, payload)
        self.assertEqual(result.stderr, 'ö')


if __name__ == '__main__':
    unittest.main()
