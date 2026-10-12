import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('http_project', Path(__file__).with_name('validate-native-http-project.py'))
http = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http)


class HttpFaultParityTests(unittest.TestCase):
    def test_engine_frames_do_not_change_http_fault_outcome(self):
        native = dict(exitCode=1, stdout='pending\n', responseHex='',
                      stderr='UserFault: HTTP callback fault\n   at Respond [instruction 1]\n   at Raven.Generated.<AsyncEntry> [instruction 1]\n')
        interpreted = dict(native, stderr='UserFault: HTTP callback fault\n   at NeoMetadata.ExportedRespond [instruction 1]\n')
        self.assertTrue(http.same_outcome(native, interpreted, fault=True))
        self.assertFalse(http.same_outcome(native, interpreted))
        for name, value in [('exitCode', 0), ('stdout', 'changed'), ('responseHex', 'ff'),
                            ('stderr', 'UserFault: another fault\n   at Respond\n'),
                            ('stderr', 'UserFault: HTTP callback fault\n'),
                            ('stderr', native['stderr'] + 'cleanup failed\n')]:
            with self.subTest(name=name, value=value):
                self.assertFalse(http.same_outcome(dict(native, **{name: value}), interpreted, fault=True))


if __name__ == '__main__':
    unittest.main()
