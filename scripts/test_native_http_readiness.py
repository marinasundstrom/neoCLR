"""Portable native HTTP readiness protocol, using real child-process pipes."""
import importlib.util
import os
import socket
import time
from pathlib import Path
import subprocess
import sys
import unittest

spec = importlib.util.spec_from_file_location('native_http', Path(__file__).with_name('verify-native-http-json.py'))
http = importlib.util.module_from_spec(spec)
spec.loader.exec_module(http)


class RetentionDiagnosticTests(unittest.TestCase):
    def test_only_documented_warnings_with_explicit_jit_roots_are_accepted(self):
        warning = ('warning: DAM2001: Mapper.Read instruction 12 -> Reader.Parse parameter 1: '
                   'cannot prove dynamic member requirement 513; annotate the incoming TypeInfo '
                   'parameter or supply explicit retention\n')
        self.assertTrue(http.expected_retention_diagnostics('', 'interpreter', None))
        self.assertTrue(http.expected_retention_diagnostics(warning, 'default', 'roots.json'))
        self.assertFalse(http.expected_retention_diagnostics(warning, 'interpreter', 'roots.json'))
        self.assertFalse(http.expected_retention_diagnostics(warning, 'default', None))
        for text in ('fault\n', warning + 'unknown warning\n', warning.replace('513', '999')):
            self.assertFalse(http.expected_retention_diagnostics(text, 'default', 'roots.json'))


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


@unittest.skipIf(os.name == 'nt', 'POSIX process groups; Windows requires target-host qualification')
class ProcessTreeTests(unittest.TestCase):
    def test_cleanup_releases_descendant_pipes_and_listener_after_launcher_exit(self):
        for exits in (False, True):
            with self.subTest(launcher_exits=exits):
                child_source = ("import socket,time; s=socket.socket(); "
                                "s.bind(('127.0.0.1',0)); s.listen(); "
                                "print(s.getsockname()[1],flush=True); time.sleep(30)")
                parent_source = ("import subprocess,sys,time; "
                                 "subprocess.Popen([sys.executable,'-u','-c'," + repr(child_source) + "]); "
                                 + ("pass" if exits else "time.sleep(30)"))
                process = http.start_process([sys.executable, '-u', '-c', parent_source],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                self.addCleanup(http.stop_process, process)
                port = int(http.read_port(process.stdout, timeout=5))
                if exits:
                    process.wait(timeout=5)
                started = time.monotonic()
                http.stop_process(process, timeout=2)
                elapsed = time.monotonic() - started
                self.assertLess(elapsed, 3)
                with socket.socket() as probe:
                    probe.bind(('127.0.0.1', port))
                print(f'Process-tree cleanup launcher_exited={exits}: {elapsed:.6f}s', flush=True)

    def test_client_timeout_kills_pipe_inheriting_descendant(self):
        source = ("import subprocess,sys,time; "
                  "subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); "
                  "time.sleep(30)")
        started = time.monotonic()
        with self.assertRaises(subprocess.TimeoutExpired):
            http.run_process([sys.executable, '-c', source], timeout=.1, text=True)
        self.assertLess(time.monotonic() - started, 3)

    def test_success_retains_output_and_exit_status(self):
        result = http.run_process([sys.executable, '-c', "print('complete')"], timeout=5, text=True)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, 'complete\n', ''))


if __name__ == '__main__':
    unittest.main()
