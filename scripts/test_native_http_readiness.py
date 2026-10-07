"""Portable native HTTP readiness protocol, using real child-process pipes."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest

spec = importlib.util.spec_from_file_location('native_http', Path(__file__).with_name('verify-native-http-json.py'))
http = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http)


class ReadinessTests(unittest.TestCase):
    def child(self, source):
        child = subprocess.Popen([sys.executable, '-u', '-c', source], stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True, encoding='utf-8')
        self.addCleanup(self.finish, child)
        return child

    @staticmethod
    def finish(child):
        if child.poll() is None:
            child.kill()
        child.wait()
        child.stdout.close()
        child.stderr.close()

    def test_port_and_remaining_output(self):
        child = self.child("print('12345'); print('Reports served')")
        self.assertEqual(http.read_port(child.stdout), '12345')
        self.assertEqual(child.stdout.read(), 'Reports served\n')
        self.assertEqual(child.wait(), 0)

    def test_exit_without_port(self):
        self.assertEqual(http.read_port(self.child('pass').stdout), '')

    def test_partial_line_has_deadline(self):
        child = self.child("import time; print('12', end='', flush=True); time.sleep(30)")
        with self.assertRaisesRegex(TimeoutError, 'report a port'):
            http.read_port(child.stdout, timeout=0.1)
        child.kill()
        child.wait()


if __name__ == '__main__':
    unittest.main()
