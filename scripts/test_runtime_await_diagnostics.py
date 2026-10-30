import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('runtime_await', Path(__file__).with_name('validate-runtime-await-sdk.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class RuntimeAwaitDiagnosticsTests(unittest.TestCase):
    def test_only_exact_jit_retention_warning_is_accepted(self):
        for mode in ('aot', 'interpreter', 'jit'):
            self.assertTrue(module.expected_diagnostics('', mode))
            self.assertEqual(module.expected_diagnostics(module.RETENTION_WARNING, mode), mode == 'jit')
            for stderr in ('fault', 'warning: unknown\n', module.RETENTION_WARNING + 'fault\n'):
                self.assertFalse(module.expected_diagnostics(stderr, mode))
