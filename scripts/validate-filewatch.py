#!/usr/bin/env python3
"""Qualify managed watcher callbacks against real OS writes in native/interpreter modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import queue
import subprocess
import sys
import threading

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'runtime/System.Runtime/test/FileSystemWatcherTests/FileSystemWatcherTests.rvnproj'


def execute(command, root, env):
    root.mkdir(parents=True)
    (root / 'nested').mkdir()
    process = subprocess.Popen([*command, str(root)], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, env=env, cwd=ROOT)
    messages = queue.Queue()
    errors = []
    def stdout():
        for line in process.stdout:
            messages.put(line)
        messages.put(None)
    def stderr():
        errors.append(process.stderr.read())
    reader = threading.Thread(target=stdout, daemon=True)
    error_reader = threading.Thread(target=stderr, daemon=True)
    reader.start()
    error_reader.start()
    lines = []
    phases = []
    try:
        while True:
            line = messages.get(timeout=180 if not lines else 30)
            if line is None:
                break
            lines.append(line)
            print(line, end='', flush=True)
            if line.startswith('READY '):
                phase = line.strip().split()[1]
                phases.append(phase)
                target = root / 'nested/settings.json'
                if phase == 'create':
                    (root / 'ignored.txt').write_text('ignored')
                    target.write_text('{"Value":1}')
                elif phase == 'replace':
                    temporary = root / 'nested/settings.tmp'
                    temporary.write_text('{"Value":2}')
                    temporary.replace(target)
                else:
                    target.write_text('{"Value":"' + phase + '"}')
        code = process.wait(timeout=5)
        error_reader.join(timeout=5)
        output = ''.join(lines)
        if code != 0 or output.count('PASS ') != 3 or phases != ['create', 'replace', 'stopped', 'restart', 'disposed']:
            raise RuntimeError(f'Watcher consumer failed ({code}): {output}\n{"".join(errors)}')
        return dict(exitCode=code, stdout=output, stderr=''.join(errors), phases=phases)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/debug' / ('neoclr.exe' if os.name == 'nt' else 'neoclr'))
    parser.add_argument('--aot', type=Path, default=ROOT / 'target/debug' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen'))
    args = parser.parse_args()
    out, bundle = args.output.resolve(), args.bundle.resolve()
    out.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ)
    report = dict(passed=False, platform=platform.platform(), cases={},
                  provider='notify 8.2.0, statically linked runtime adapter',
                  sources={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in [*PROJECT.parent.rglob('*.rvn'), PROJECT,
                                     ROOT / 'runtime/System.Runtime/src/System.Runtime/System/IO/FileSystemWatcher.rvn']})
    try:
        build = out / 'native'
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'),
                                 '--profile', 'windows-http' if os.name == 'nt' else 'http',
                                 '--project', str(PROJECT), '--bundle', str(bundle),
                                 '--aot', str(args.aot.resolve()), '--output', str(build)],
                                capture_output=True, text=True, cwd=ROOT, timeout=600)
        (out / 'build.log').write_text(result.stdout + result.stderr)
        if result.returncode:
            raise RuntimeError('Build failed; see ' + str(out / 'build.log'))
        receipt = json.loads((build / 'build.json').read_text())
        command = next(item['command'] for item in receipt['commands']
                       if item['phase'] == 'host-compile-link')[:]
        ownership = out / ('ownership.exe' if os.name == 'nt' else 'ownership')
        harness = PROJECT.parent / 'native-ownership.c'
        for index, argument in enumerate(command):
            if Path(argument).name in ('http-host.c', 'windows-console-host.c'):
                command[index] = str(harness)
            elif argument.startswith('/Fe:'):
                command[index] = '/Fe:' + str(ownership)
            elif index and command[index - 1] == '-o':
                command[index] = str(ownership)
        compiled = subprocess.run(command, capture_output=True, text=True, cwd=ROOT, timeout=120)
        (out / 'ownership-build.log').write_text(compiled.stdout + compiled.stderr)
        if compiled.returncode:
            raise RuntimeError('Native ownership harness build failed')
        owned = subprocess.run([str(ownership), str(out)], capture_output=True, text=True, timeout=30)
        report['cases']['ownership'] = dict(exitCode=owned.returncode, stdout=owned.stdout, stderr=owned.stderr)
        if owned.returncode:
            raise RuntimeError('Native watcher ownership/teardown failed')
        native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
        report['cases']['native'] = execute(native, out / 'native-files', env)
        lib = bundle / 'lib'
        interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                       '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '100000000']
        for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
            interpreter += ['--module', str(lib / (name + '.dll'))]
        report['cases']['interpreter'] = execute([*interpreter, '--'], out / 'interpreter-files', env)
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
