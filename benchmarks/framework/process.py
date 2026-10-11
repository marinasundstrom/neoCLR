"""Bounded subprocess transport shared by benchmark controllers."""
import os
import queue
import signal
import subprocess
import threading
import time
import tempfile

class Guest:
    def __init__(self, command, timeout=120):
        self.timeout = timeout
        self.error_file = tempfile.TemporaryFile()
        self.started = time.perf_counter()
        try:
            self.process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=self.error_file,
                                            start_new_session=True)
        except Exception:
            self.error_file.close()
            raise
        self.lines = queue.Queue()
        self.output = []
        def read():
            for line in self.process.stdout:
                self.output.append(line)
                self.lines.put((line.rstrip(b'\r\n'), time.perf_counter()))
            self.lines.put((None, time.perf_counter()))
        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def line(self, expected=None):
        try:
            line, at = self.lines.get(timeout=self.timeout)
        except queue.Empty as error:
            raise TimeoutError("Timed out waiting for benchmark worker output") from error
        if line is None or expected is not None and line != expected:
            detail = self.errors().decode(errors='replace') if line is None else ''
            raise RuntimeError(f'Unexpected guest output: {line!r}; expected {expected!r}; {detail}')
        return line, at

    def errors(self):
        self.error_file.seek(0)
        return self.error_file.read(8192)

    def finish(self):
        code = self.process.wait(timeout=self.timeout)
        self.reader.join(timeout=5)
        error = self.errors()
        if code or error or self.reader.is_alive():
            raise RuntimeError(f'Guest failed: {code}, {error!r}, {self.output!r}')

    def close(self):
        if self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.process.wait(timeout=10)
        self.reader.join(timeout=5)
        self.process.stdout.close()
        self.error_file.close()
