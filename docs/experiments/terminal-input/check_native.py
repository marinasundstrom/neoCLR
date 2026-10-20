#!/usr/bin/env python3
"""Build/check the private native POSIX transport against real pseudo-terminals."""
import argparse
import json
import os
from pathlib import Path
import pty
import select
import subprocess
import termios
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--jit-host', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
args.output.mkdir(parents=True, exist_ok=True)
binary = (args.output / 'terminal-native').resolve()
command = ['cc', '-pthread', '-std=c11', '-D_POSIX_C_SOURCE=200809L', '-Wall', '-Wextra', '-Werror',
           '-I', str(root / 'docs/experiments/aot-console'),
           str(root / 'docs/experiments/terminal-input/native.c'),
           str(root / 'docs/experiments/aot-console/console.c'), '-o', str(binary)]
subprocess.run(command, check=True)
rows = []
for case, data, expected in [('redirected', b'a', b'PASS redirected\n'),
                             ('bytes', bytes(range(256)), b'PASS all bytes\n')]:
    result = subprocess.run([binary, case], input=data, capture_output=True, timeout=5)
    assert result.returncode == 0, result.stderr
    assert result.stdout == expected, result.stdout
    rows.append(dict(case=case, passed=True))
for case in ['switch', 'keys', 'hangup', 'replace', 'scope']:
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    process = subprocess.Popen([binary, case], stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        if case == 'switch':
            os.write(master, b'abcd\n')
        elif case not in ('replace', 'scope'):
            ready = b''
            deadline = time.monotonic() + 5
            while b'READY\n' not in ready:
                assert time.monotonic() < deadline, ready
                if select.select([process.stdout], [], [], 0.1)[0]:
                    chunk = os.read(process.stdout.fileno(), 4096)
                    assert chunk, (process.poll(), process.stderr.read())
                    ready += chunk
            if case == 'hangup':
                os.close(master)
                master = None
            else:
                for part in [b'\x03\xc3', b'\xa9\x1b', b'[A\r\n']:
                    os.write(master, part)
                    time.sleep(0.01)
        stdout, stderr = process.communicate(timeout=5)
        assert process.returncode == 0, (case, stdout, stderr)
        expected = {'switch': b'PASS no read-ahead\n', 'keys': b'PASS native terminal\n', 'hangup': b'PASS hangup\n', 'replace': b'PASS replaced stdin\n', 'scope': b'PASS native scope\n'}[case]
        assert stdout == expected, (case, stdout)
        if case != 'hangup':
            restored = termios.tcgetattr(slave)
            original[3] &= ~getattr(termios, 'PENDIN', 0)
            restored[3] &= ~getattr(termios, 'PENDIN', 0)
            assert original == restored, case
        rows.append(dict(case=case, passed=True))
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        if master is not None:
            os.close(master)
        os.close(slave)
entry_binary = (args.output / 'terminal-entry').resolve()
entry_command = command.copy()
entry_command[entry_command.index(str(root / 'docs/experiments/terminal-input/native.c'))] = str(root / 'docs/experiments/terminal-input/native-entry.c')
entry_command += [str(root / 'tools/native/console-host.c')]
entry_command[entry_command.index('-o') + 1] = str(entry_binary)
subprocess.run(entry_command, check=True)
entry_binaries = [('entry', entry_binary)]
if args.jit_host:
    jit_binary = (args.output / 'terminal-jit-entry').resolve()
    jit_command = command.copy()
    jit_command[jit_command.index(str(root / 'docs/experiments/terminal-input/native.c'))] = str(root / 'docs/experiments/terminal-input/native-entry.c')
    jit_command.remove(str(root / 'docs/experiments/aot-console/console.c'))
    jit_command += ['-DNEOCLR_TEST_REAL_HOST', '-DNEOCLR_TEST_JIT', str(args.jit_host.resolve()),
                    '-Wl,-rpath,' + str(args.jit_host.resolve().parent)]
    jit_command[jit_command.index('-o') + 1] = str(jit_binary)
    subprocess.run(jit_command, check=True)
    entry_binaries.append(('jit-entry', jit_binary))
for label, current_binary, fault in [(label, binary, fault) for label, binary in entry_binaries for fault in [False, True]]:
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    environment = os.environ.copy()
    environment.pop('NEOCLR_TEST_TERMINAL_FAULT', None)
    if fault:
        environment['NEOCLR_TEST_TERMINAL_FAULT'] = '1'
    try:
        result = subprocess.run([current_binary], stdin=slave, capture_output=True, env=environment, timeout=5)
        assert result.returncode == (1 if fault else 0), result.stderr
        restored = termios.tcgetattr(slave)
        original[3] &= ~getattr(termios, 'PENDIN', 0)
        restored[3] &= ~getattr(termios, 'PENDIN', 0)
        assert original == restored
        rows.append(dict(case=label + ('-fault' if fault else '-success'), passed=True))
    finally:
        os.close(master)
        os.close(slave)
report = dict(passed=True, platform=os.uname().sysname, buildCommand=command, entryBuildCommand=entry_command, cases=rows)
(args.output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(rows, indent=2))
