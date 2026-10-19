#!/usr/bin/env python3
"""Exercise the Rust host diagnostic against a POSIX PTY and redirected input."""
import argparse
import os
import re
import selectors
import subprocess
import termios
import time

parser = argparse.ArgumentParser()
parser.add_argument("--program", default="target/debug/examples/terminal_input")
args = parser.parse_args()
master, slave = os.openpty()
original = termios.tcgetattr(slave)
process = None
try:
    process = subprocess.Popen(
        [args.program], stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    output = bytearray()
    deadline = time.monotonic() + 60
    with selectors.DefaultSelector() as selector:
        selector.register(process.stdout, selectors.EVENT_READ)
        while b"Ctrl+C closes and restores the terminal." not in output:
            if time.monotonic() >= deadline:
                raise AssertionError("terminal acquisition timeout")
            if selector.select(0.1):
                part = os.read(process.stdout.fileno(), 4096)
                if not part:
                    raise AssertionError("diagnostic exited before acquiring input")
                output.extend(part)
    assert b"terminal_input: true" in output, output
    os.write(master, "é".encode("utf-8") + b"\x03")
    tail, error = process.communicate(timeout=20)
    output.extend(tail)
    assert process.returncode == 0, (output, error)
    payload = bytearray()
    for chunk in re.findall(rb"\[([0-9a-f]{2}(?:, [0-9a-f]{2})*)\]", output):
        payload.extend(int(value, 16) for value in chunk.split(b", "))
    assert payload == "é".encode("utf-8") + b"\x03", output
    restored = termios.tcgetattr(slave)
    # BSD sets PENDIN internally when canonical mode resumes, even after an
    # exact tcsetattr restoration. It is pending queue state, not a mode choice.
    restored[3] &= ~getattr(termios, "PENDIN", 0)
    original[3] &= ~getattr(termios, "PENDIN", 0)
    assert restored == original, "terminal settings not restored"
finally:
    if process is not None and process.poll() is None:
        process.kill()
        process.communicate()
    # Keep the test runner's device clean even after a diagnostic failure.
    termios.tcsetattr(slave, termios.TCSANOW, original)
    os.close(master)
    os.close(slave)

redirected = subprocess.run(
    [args.program], stdin=subprocess.DEVNULL, capture_output=True, timeout=20
)
assert redirected.returncode != 0, redirected.stdout
assert b"terminal_input: false" in redirected.stdout, redirected.stdout
assert b"Unsupported" in redirected.stderr, redirected.stderr
print("PASS: PTY UTF-8/control input, explicit restoration, redirected-input rejection")
