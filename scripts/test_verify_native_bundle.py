import importlib.util
from pathlib import Path
import subprocess
import unittest

spec = importlib.util.spec_from_file_location('verify_bundle', Path(__file__).with_name('verify-native-bundle.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class SampleOutputTests(unittest.TestCase):
    def test_known_jit_retention_diagnostic_is_retained_without_accepting_other_errors(self):
        def result(code=0, stdout='42\n', stderr=''):
            return subprocess.CompletedProcess(['runtime'], code, stdout, stderr)
        self.assertTrue(module.sample_output_matches(result(), '42\n'))
        warning = module.RETENTION_WARNING
        self.assertFalse(module.sample_output_matches(result(stderr=warning), '42\n'))
        self.assertTrue(module.sample_output_matches(result(stderr=warning), '42\n', True))
        for value in (result(code=1, stderr=warning), result(stdout='wrong', stderr=warning),
                      result(stderr=warning + 'fault\n'), result(stderr='warning: unknown\n')):
            self.assertFalse(module.sample_output_matches(value, '42\n', True))
