"""The HTTP kit's fault control must execute independently of response validation."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('kit', Path(__file__).with_name('verify-native-build-kit.py'))
kit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(kit)


class HttpFaultInjectionTests(unittest.TestCase):
    def test_current_handler_injects_before_normal_route_rejection(self):
        source = (kit.ROOT / 'docs/experiments/http-server/Server.rvn').read_text()
        updated = kit.inject_http_handler_fault(source)
        self.assertEqual(updated.count('System.Fail("HTTP handler failed")'), 1)
        self.assertLess(updated.index('System.Fail("HTTP handler failed")'),
                        updated.index('if request.Method.Method'))
        self.assertIn('return Task.FromResult(result)', updated)

    def test_fault_contract_allows_engine_specific_frames(self):
        vm = dict(exitCode=1, stdout='pending\n', responseHex='',
                  stderr='UserFault: HTTP handler failed\n   at CLI.Name [instruction 6]\n')
        native = {**vm, 'stderr': 'UserFault: HTTP handler failed\n   at Respond [instruction 6]\n'}
        kit.verify_http_handler_fault(vm, native)
        for field, value in [('exitCode', 0), ('stdout', 'different'),
                             ('responseHex', '00'), ('stderr', 'UserFault: other\n'),
                             ('stderr', 'UserFault: HTTP handler failed\n')]:
            with self.subTest(field=field, value=value), self.assertRaises(AssertionError):
                kit.verify_http_handler_fault(vm, {**native, field: value})

    def test_missing_or_ambiguous_handler_rejects(self):
        source = (kit.ROOT / 'docs/experiments/http-server/Server.rvn').read_text()
        for invalid in ('', source.replace('func Respond(', 'func Handle('), source + source):
            with self.subTest(source=invalid[:30]), self.assertRaises(ValueError):
                kit.inject_http_handler_fault(invalid)


if __name__ == '__main__':
    unittest.main()
