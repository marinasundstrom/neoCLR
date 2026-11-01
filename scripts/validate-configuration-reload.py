#!/usr/bin/env python3
"""Qualify opt-in JSON reload, live DI consumers, errors and disposal with real files."""
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
PROJECT = ROOT / 'extensions/Neo.Configuration/test/Neo.Configuration.Reload.Tests/Neo.Configuration.Reload.Tests.rvnproj'


def execute(command, root, env, mode):
    root.mkdir(parents=True)
    (root / 'appsettings.json').write_text('{"Greeting":{"Name":"before"}}')
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
                target = root / 'appsettings.json'
                if phase == 'malformed':
                    target.write_text('{invalid')
                elif phase == 'deleted':
                    target.unlink()
                elif phase == 'optional-created':
                    (root / 'optional.json').write_text('{"Value":"created"}')
                elif phase == 'optional-deleted':
                    (root / 'optional.json').unlink()
                else:
                    text = '{"Greeting":{"Name":"' + phase + '"}}'
                    if phase == 'replaced':
                        temporary = root / 'appsettings.tmp'
                        temporary.write_text(text)
                        temporary.replace(target)
                    else:
                        target.write_text(text)
        code = process.wait(timeout=5)
        error_reader.join(timeout=5)
        output = ''.join(lines)
        transcript = ''.join(line for line in lines if not line.startswith('READY '))
        expected = ('PASS JSON reload is opt in and updates DI consumers while preserving valid data\n'
                    'Tests: 1, passed: 1, failed: 0, skipped: 0\n')
        stderr = ''.join(errors)
        warning = 'warning: DAM2004: dynamic-dispatch implementation bodies are not analyzed; reflection inside them may require explicit retention\n'
        diagnostic = warning if mode == 'jit' and stderr == warning else ''
        if code != 0 or transcript != expected or stderr != diagnostic or phases != ['changed', 'malformed', 'replaced', 'deleted', 'optional-created', 'optional-deleted', 'disposed']:
            raise RuntimeError(f'Configuration reload consumer failed ({code}): {output}\n{"".join(errors)}')
        return dict(exitCode=code, stdout=output, stderr=stderr, compilerDiagnostic=diagnostic, phases=phases)
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
                                     ROOT / 'runtime/System.Runtime/src/System.Runtime/System/IO/FileSystemWatcher.rvn',
                                     ROOT / 'extensions/Neo.Configuration/src/Neo.Configuration.Json/Neo/Configuration/Json/JsonConfiguration.rvn']})
    report['frameworkSources'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in (ROOT / 'tools/testing/NeoCLR.Testing/NeoClr/Testing').glob('*.rvn')}
    report['validatorSha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    report['runtimeSha256'] = hashlib.sha256(args.runtime.resolve().read_bytes()).hexdigest()
    report['aotSha256'] = hashlib.sha256(args.aot.resolve().read_bytes()).hexdigest()
    report['bundleManifestSha256'] = hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest()
    jit_host = bundle / 'bin' / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib')
    report['jitHostSha256'] = hashlib.sha256(jit_host.read_bytes()).hexdigest()
    try:
        build = out / 'native'
        result = subprocess.run([sys.executable, str(ROOT / 'scripts/build-native-project.py'),
                                 '--profile', 'windows-http' if os.name == 'nt' else 'http',
                                 '--project', str(PROJECT), '--bundle', str(bundle),
                                 '--aot', str(args.aot.resolve()), '--output', str(build)],
                                capture_output=True, text=True, cwd=ROOT, timeout=600)
        (out / 'build.log').write_text(result.stdout + result.stderr)
        if result.returncode:
            raise RuntimeError('Build failed; see ' + str(out / 'build.log') + '\n' + (result.stdout + result.stderr)[-12000:])
        native = [str(build / ('app.exe' if os.name == 'nt' else 'app'))]
        report['cases']['native'] = execute(native, out / 'native-files', env, 'native')
        lib = bundle / 'lib'
        interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll'), '--execution', 'interpreter', '--system', str(lib / 'System.runtime.neox'),
                       '--object-root', str(lib / 'System.Runtime.dll'), '--instructions', '100000000']
        for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web', 'Neo.DependencyInjection.Abstractions', 'Neo.DependencyInjection', 'Neo.Configuration.Abstractions', 'Neo.Configuration', 'Neo.Configuration.Json'):
            interpreter += ['--module', str(lib / (name + '.dll'))]
        report['cases']['interpreter'] = execute([*interpreter, '--'], out / 'interpreter-files', env, 'interpreter')
        jit = interpreter[:]
        jit[jit.index('--execution') + 1] = 'jit'
        limit = jit.index('--instructions')
        del jit[limit:limit + 2]
        jit += ['--jit-host', str(jit_host), '--jit-compiler', str(args.aot.resolve()), '--']
        report['cases']['jit'] = execute(jit, out / 'jit-files', env, 'jit')
        report['passed'] = True
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
