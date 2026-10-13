import os
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
from process import Guest


class WorkerCleanup(unittest.TestCase):
    def test_windows_terminates_owned_process_tree(self):
        guest = Guest.__new__(Guest)
        guest.process = Mock(pid=1234)
        guest.process.poll.return_value = None
        guest.reader = Mock()
        guest.error_file = Mock()
        with patch('process.os.name', 'nt'), patch('process.subprocess.run') as run:
            guest.close()
            self.assertEqual(run.call_args.args[0], ['taskkill', '/PID', '1234', '/T', '/F'])
        guest.process.wait.assert_called_once_with(timeout=10)
        guest.error_file.close.assert_called_once()

    def test_native_worker_cleanup(self):
        guest = Guest([sys.executable, '-c', 'import time; print("READY", flush=True); time.sleep(60)'])
        guest.line(b'READY')
        guest.close()
        self.assertIsNotNone(guest.process.poll())
        self.assertFalse(guest.reader.is_alive())
