#!/usr/bin/env python3
"""Focused sanitizer-backed protocol/lifetime test, independent of Raven."""
import importlib.util
import json
import os
from pathlib import Path
import select
import socket
import shlex
import subprocess
import sys
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('profiler', ROOT / 'tools/profiler/neoclr-profiler.py')
profiler = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profiler)


class AttachTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory(prefix='nc-build-')
        cls.binary = str(Path(cls.build.name) / 'host')
        native = HERE.parent / 'aot-console'
        subprocess.run([*shlex.split(os.environ.get('CC', 'clang')),
                        *shlex.split(os.environ.get('CFLAGS', '')), '-std=c11', '-g', '-O1', '-Wall', '-Wextra', '-Werror',
                        '-fsanitize=undefined,bounds', '-DNEOCLR_GC_PROFILE', '-DNEOCLR_DIAGNOSTICS',
                        str(HERE / 'host-test.c'), str(HERE / 'attach.c'), str(native / 'native-gc.c'),
                        str(native / 'root-probe.c'), '-pthread', '-o', cls.binary], check=True)

    @classmethod
    def tearDownClass(cls):
        cls.build.cleanup()

    def line(self, stream):
        self.assertTrue(select.select([stream], [], [], 5)[0], 'host output timeout')
        return stream.readline()

    def test_lifecycle(self):
        # Short private path also fits macOS's sockaddr_un limit.
        with tempfile.TemporaryDirectory(prefix='nc-', dir='/tmp') as directory:
            env = dict(os.environ, NEOCLR_DIAGNOSTICS_DIRECTORY=directory)
            process = subprocess.Popen([self.binary], env=env, stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            endpoint = f'{directory}/neoclr-{process.pid}.sock'
            try:
                self.assertEqual(self.line(process.stdout), 'ready\n')
                self.assertEqual(os.stat(endpoint).st_mode & 0o777, 0o600)
                empty = profiler.snapshot(endpoint)
                self.assertFalse(empty['available'])
                self.assertFalse(empty['fresh'])
                process.stdin.write('c'); process.stdin.flush()
                self.assertEqual(self.line(process.stdout), 'collected\n')
                stale = profiler.snapshot(endpoint)
                self.assertTrue(stale['available'])
                self.assertFalse(stale['fresh'])
                self.assertEqual(stale['allocatedPayloadBytes'], 16)
                self.assertEqual(stale['allocatedPayloadBytes'], stale['reclaimedPayloadBytes'])
                self.assertEqual(sum(stale['gcPauseBuckets']), stale['gcAttempts'])

                rows = []
                thread = threading.Thread(target=lambda: rows.append(profiler.snapshot(endpoint)))
                thread.start()
                # Continue owner work until the reader completes; no runtime service on this thread.
                for _ in range(200):
                    if not thread.is_alive(): break
                    process.stdin.write('c'); process.stdin.flush()
                    self.assertEqual(self.line(process.stdout), 'collected\n')
                    time.sleep(.002)
                thread.join(3)
                self.assertFalse(thread.is_alive())
                self.assertTrue(rows[0]['fresh'])
                self.assertGreater(rows[0]['sequence'], stale['sequence'])
                self.assertGreater(rows[0]['allocations'], stale['allocations'])
                self.assertEqual(rows[0]['instance'], stale['instance'])
                for request in (b'NEOCLR/2 SNAPSHOT\n', b'NEOCLR/1 PAUSE\n', b'x' * 100):
                    with socket.socket(socket.AF_UNIX) as bad:
                        bad.settimeout(2); bad.connect(endpoint); bad.sendall(request)
                        self.assertIn(b'unsupported-request', bad.recv(4096))
                # Slow and abruptly disconnected readers cannot strand or terminate the runtime.
                with socket.socket(socket.AF_UNIX) as slow:
                    slow.connect(endpoint)
                    process.stdin.write('c'); process.stdin.flush()
                    self.assertEqual(self.line(process.stdout), 'collected\n')
                    self.assertTrue(profiler.snapshot(endpoint)['available'])
                with socket.socket(socket.AF_UNIX) as gone:
                    gone.connect(endpoint); gone.sendall(b'NEOCLR/1 SNAPSHOT\n')
                cli = subprocess.run([sys.executable, str(ROOT / 'tools/profiler/neoclr-profiler.py'),
                                      '--socket', endpoint, '--count', '2', '--interval', '0'],
                                     capture_output=True, text=True, timeout=5)
                self.assertEqual(cli.returncode, 0, cli.stderr)
                self.assertEqual(len(cli.stdout.splitlines()), 2)
                process.stdin.write('q'); process.stdin.flush()
                stdout, stderr = process.communicate(timeout=5)
                self.assertEqual((process.returncode, stderr), (0, ''))
                self.assertFalse(Path(endpoint).exists())
                with self.assertRaises(OSError): profiler.snapshot(endpoint)
            finally:
                if process.poll() is None: process.kill(); process.communicate()

    def test_disabled_and_invalid(self):
        env = dict(os.environ)
        env.pop('NEOCLR_DIAGNOSTICS_DIRECTORY', None)
        disabled = subprocess.run([self.binary], input='cq', env=env, capture_output=True, text=True, timeout=5)
        self.assertEqual((disabled.returncode, disabled.stderr), (0, ''))
        with tempfile.TemporaryDirectory(prefix='nc-', dir='/tmp') as directory:
            os.chmod(directory, 0o755)
            invalid = subprocess.run([self.binary], env=dict(env, NEOCLR_DIAGNOSTICS_DIRECTORY=directory),
                                     capture_output=True, text=True, timeout=5)
            self.assertEqual(invalid.returncode, 78)
            self.assertIn('endpoint unavailable', invalid.stderr)


if __name__ == '__main__': unittest.main()
