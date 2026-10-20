#!/usr/bin/env python3
"""Exercise public Raven ReadKey with real PTYs in all three execution engines."""
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
parser.add_argument('bundle', type=Path)
parser.add_argument('application', type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--any-key', type=Path)
parser.add_argument('--ctrl-c', type=Path)
parser.add_argument('--case', choices=['redirected', 'text-first', 'keys', 'handoff', 'fault', 'any-key', 'ctrl-c'])
args = parser.parse_args()
if args.case == "any-key" and not args.any_key:
    parser.error("--case any-key requires --any-key")
if args.case == "ctrl-c" and not args.ctrl_c:
    parser.error("--case ctrl-c requires --ctrl-c")
bundle, app = args.bundle.resolve(), args.application.resolve()
common = ['--system', str(bundle / 'lib/System.runtime.neox'), '--object-root', str(bundle / 'lib/System.Runtime.dll')]
for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'System.Data.PostgreSql']:
    common += ['--module', str(bundle / 'lib' / (name + '.dll'))]
rows = []
for engine in ['interpreter', 'jit', 'aot']:
    base = [str(app / 'app')] if engine == 'aot' else [str(bundle / 'bin/neoclr'), 'run', str(app / 'app.dll'), '--execution', engine, *common]
    if engine == 'interpreter':
        base += ['--instructions', '100000000']
    if engine == 'jit':
        base += ['--jit-compiler', str(bundle / 'bin/neoclr-codegen'), '--jit-host', str(bundle / 'bin/neoclr-jit-host.dylib')]
    for case in ['redirected', 'text-first', 'keys', 'handoff', 'fault'] + (['any-key'] if args.any_key else []) + (['ctrl-c'] if args.ctrl_c else []):
        if args.case and case != args.case:
            continue
        command = base + ([case] if engine == 'aot' else ['--', case])
        if case in ['any-key', 'ctrl-c']:
            example = args.any_key if case == 'any-key' else args.ctrl_c
            command = [value.replace(str(app), str(example.resolve())) for value in base]
        if case in ['redirected', 'text-first']:
            result = subprocess.run(command, input=b'abc\n', capture_output=True, timeout=120)
            assert result.returncode == 0 and result.stdout == b'PASS\n', (engine, case, result.stdout, result.stderr)
        else:
            master, slave = pty.openpty()
            original = termios.tcgetattr(slave)
            process = subprocess.Popen(command, stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            output = b''
            def until(marker):
                global output
                deadline = time.monotonic() + 120
                while marker not in output:
                    assert time.monotonic() < deadline, (engine, case, output)
                    if select.select([process.stdout], [], [], 0.1)[0]:
                        chunk = os.read(process.stdout.fileno(), 4096)
                        assert chunk, (engine, case, process.poll(), output, process.stderr.read())
                        output += chunk
            try:
                if case in ['any-key', 'ctrl-c']:
                    prompt = b'Press any key to exit.\n' if case == 'any-key' else b'Press Ctrl+C to exit.\n'
                    until(prompt)
                    if case == 'ctrl-c':
                        os.write(master, b'a')
                        time.sleep(.1)
                        assert process.poll() is None, 'ordinary key must not exit Ctrl+C sample'
                    os.write(master, b'\x03')
                if case == 'handoff':
                    until(b'READY\n')
                    os.write(master, b'\xc3z')
                    until(b'TEXT\n')
                    os.write(master, b'tail\n')
                if case == 'keys':
                    until(b'READY\n')
                    # Slow split Unicode must not inherit the Escape timeout.
                    for part, delay in [(b'a\x03\x1b[1;6D\xc3', .07), (b'\xa9\xf0\x9f', .07),
                                        (b'\x98\x80\x1bx', .01), (b'\x1b', .10),
                                        (b'\x1b[99~\xc3z', 0)]:
                        os.write(master, part)
                        time.sleep(delay)
                    until(b'TEXT\n')
                    os.write(master, b'tail\n')
                stdout, stderr = process.communicate(timeout=120)
                output += stdout
                assert process.returncode == (1 if case == 'fault' else 0), (engine, case, output, stderr)
                if case == 'fault':
                    assert b'key session fault cleanup' in stderr, stderr
                elif case in ['any-key', 'ctrl-c']:
                    assert output == prompt, output
                else:
                    assert output.endswith(b'PASS\n'), (output, stderr)
                restored = termios.tcgetattr(slave)
                original[3] &= ~getattr(termios, 'PENDIN', 0)
                restored[3] &= ~getattr(termios, 'PENDIN', 0)
                assert original == restored, (engine, case, 'terminal not restored')
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()
                os.close(master)
                os.close(slave)
        rows.append(dict(engine=engine, case=case, passed=True, command=command))
        print(engine, case, 'PASS', flush=True)
args.output.write_text(json.dumps(dict(passed=True, cases=rows), indent=2) + '\n')
