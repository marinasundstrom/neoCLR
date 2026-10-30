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

    def test_guest_diagnostics_require_explicit_exact_allowance_and_success(self):
        import sys
        warning = module.RETENTION_WARNING.encode()
        for code, stderr, allowed, accepted in (
            (0, b'', (b'',), True), (0, warning, (b'',), False),
            (0, warning, (b'', warning), True), (1, warning, (b'', warning), False),
            (0, warning + b'fault', (b'', warning), False)):
            guest = module.Guest([sys.executable, '-c',
                f'import sys; sys.stderr.buffer.write({stderr!r}); sys.exit({code})'])
            try:
                if accepted:
                    self.assertEqual(guest.finish(allowed_stderr=allowed), stderr)
                else:
                    with self.assertRaises(RuntimeError): guest.finish(allowed_stderr=allowed)
            finally: guest.close()
