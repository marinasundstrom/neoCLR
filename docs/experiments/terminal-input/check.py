#!/usr/bin/env python3
"""POSIX PTY allocation/restoration check for the terminal_input Rust example."""
import os
import pty
import select
import subprocess
import sys
import termios
import time


def main():
    binary = sys.argv[1] if len(sys.argv) > 1 else 'target/release/examples/terminal_input'
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    process = subprocess.Popen([binary], stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def until(token):
        data = b''
        deadline = time.monotonic() + 5
        while token not in data:
            if time.monotonic() >= deadline:
                raise AssertionError((token, data))
            if select.select([process.stdout], [], [], 0.1)[0]:
                chunk = os.read(process.stdout.fileno(), 4096)
                if not chunk:
                    raise AssertionError((process.poll(), data))
                data += chunk
        return data

    try:
        output = until(b'Type to inspect')
        for fragment in [b'\xc3', b'\xa9', b'\x1b', b'[A', b'\x03']:
            os.write(master, fragment)
            output += until(b'read allocations: 0')
        process.wait(timeout=5)
        assert process.returncode == 0, process.stderr.read()
        assert output.count(b'read allocations: 0') == 5, output
        restored = termios.tcgetattr(slave)
        original[3] &= ~getattr(termios, 'PENDIN', 0)
        restored[3] &= ~getattr(termios, 'PENDIN', 0)
        assert restored == original
        print(output.decode(), end='')
        print('PASS: five fragmented reads, zero allocator calls each, restored terminal')
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)
        os.close(slave)


if __name__ == '__main__':
    main()
