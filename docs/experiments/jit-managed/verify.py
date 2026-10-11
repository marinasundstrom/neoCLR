#!/usr/bin/env python3
"""Qualify the synchronous JIT against an existing, successful console AOT build."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--jit', type=Path, default=ROOT / 'tools/aot-poc/target/release/neoclr-aot-poc')
    parser.add_argument('--runtime', type=Path, default=ROOT / 'target/release/neoclr')
    args = parser.parse_args()
    build, out = args.build.resolve(), args.output.resolve()
    report = json.loads((build / 'build.json').read_text())
    if not report['passed'] or report['profile'] != 'macos-arm64-console-v1':
        raise ValueError('Requires a successful macOS ARM64 console build')
    if sha(build / 'app.dll') != report['artifacts']['app.dll'] or sha(build / 'app') != report['artifacts']['app']:
        raise ValueError('Qualified application artifacts changed')
    out.mkdir(parents=True, exist_ok=False)
    host = out / 'host.dylib'
    link = next(c['command'] for c in report['commands'] if c.get('phase') == 'host-compile-link')
    compile_command = next(c['command'] for c in report['commands'] if c.get('phase') == 'native-compilation')
    command = []
    for item in link:
        if item == str(build / 'app.o'):
            continue
        if item.endswith('/tools/native/console-host.c'):
            item = str(Path(__file__).with_name('host.c'))
        if item == str(build / 'app.pending'):
            item = str(host)
        command.append(item)
    command += [str(ROOT / 'docs/experiments/aot-console/task-queue.c'), str(ROOT / 'docs/experiments/aot-console/socket-listener.c'), '-dynamiclib']
    result = subprocess.run(command, capture_output=True, text=True, timeout=180)
    (out / 'host-build.log').write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError('Host build failed; see host-build.log')
    invocation = [str(args.jit.resolve()), '--jit-managed', *compile_command[1:]]
    invocation[invocation.index(str(build / 'app.o'))] = str(host)
    interpreter = [str(args.runtime.resolve()), 'run', str(build / 'app.dll')]
    for flag in ['--system', '--module', '--object-root']:
        for i, item in enumerate(compile_command):
            if item == flag:
                interpreter += [flag, compile_command[i + 1]]
    interpreter += ['--instructions', '100000000']
    evidence = dict(passed=False, buildReportSha256=sha(build / 'build.json'),
                    appSha256=sha(build / 'app.dll'), hostSha256=sha(host),
                    jitSha256=sha(args.jit), interpreterSha256=sha(args.runtime),
                    hostCommand=command, jitCommand=invocation, cases=[])
    try:
        for label, arguments in [('suite', []), ('no-match', ['--filter', 'no-test-has-this-name']),
                                  ('bounds-fault', ['--guard', 'list-end'])]:
            runs = {}
            for mode, cmd in [('aot', [str(build / 'app'), *arguments]),
                              ('interpreter', [*interpreter, '--', *arguments]),
                              ('jit', [*invocation, *[part for arg in arguments for part in ['--jit-entry-argument', arg]]])]:
                started = time.perf_counter()
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
                record = dict(label=label, mode=mode, processExitCode=result.returncode,
                              stdout=result.stdout, stderr=result.stderr,
                              processSeconds=time.perf_counter() - started)
                if mode == 'jit':
                    lines = result.stderr.splitlines(keepends=True)
                    summaries = [line for line in lines if line.startswith('JIT_RESULT ')]
                    if result.returncode or len(summaries) != 1:
                        raise RuntimeError('JIT failed: ' + result.stderr)
                    record['jit'] = json.loads(summaries[0].removeprefix('JIT_RESULT '))
                    record['exitCode'] = record['jit']['guestExitCode']
                    record['stderr'] = ''.join(line for line in lines if not line.startswith('JIT_RESULT '))
                else:
                    record['exitCode'] = result.returncode
                evidence['cases'].append(record)
                runs[mode] = record
            if label != 'bounds-fault':
                outputs = {(r['exitCode'], r['stdout'], r['stderr']) for r in runs.values()}
                if len(outputs) != 1 or runs['jit']['exitCode'] != (0 if label == 'suite' else 2):
                    raise RuntimeError('Test runner outcomes differ: ' + label)
            else:
                if any(not r['exitCode'] or r['stdout'] or 'ArrayList index out of range' not in r['stderr'] for r in runs.values()):
                    raise RuntimeError('Bounds fault contract differs')
        # Missing required helpers must fail during binding, before host invocation.
        incomplete_source = out / 'incomplete-host.c'
        incomplete_source.write_text('#include <stdlib.h>\nint neoclr_jit_run_v1(const void *p, int k, int c, const char **v) { (void)p; (void)k; (void)c; (void)v; abort(); }\n')
        incomplete = out / 'incomplete.dylib'
        subprocess.run([link[0], '-dynamiclib', str(incomplete_source), '-o', str(incomplete)], check=True, capture_output=True, timeout=60)
        negative = [str(incomplete) if part == str(host) else part for part in invocation]
        result = subprocess.run(negative, capture_output=True, text=True, timeout=180)
        if result.returncode != 1 or result.stdout or 'JIT_RESULT ' in result.stderr or 'symbol not found' not in result.stderr:
            raise RuntimeError('Missing-helper rejection did not precede execution: ' + result.stderr)
        evidence['missingHelperRejected'] = True
        evidence['missingHelperDiagnostic'] = result.stderr
        # Shared lowering must retain the existing AOT object for this exact input.
        object_path = out / 'regression.o'
        aot_command = [str(args.jit.resolve()), *compile_command[1:]]
        aot_command[aot_command.index(str(build / 'app.o'))] = str(object_path)
        result = subprocess.run(aot_command, capture_output=True, text=True, timeout=180)
        if result.returncode:
            raise RuntimeError('AOT regression compilation failed: ' + result.stderr)
        evidence['aotObjectSha256'] = sha(object_path)
        evidence['previousAotObjectSha256'] = report['artifacts']['app.o']
        if evidence['aotObjectSha256'] != evidence['previousAotObjectSha256']:
            raise RuntimeError('AOT object changed; inspect before accepting the shared-lowering refactor')
        evidence['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print('PASS managed JIT/AOT/interpreter suite, filtering and bounds fault')


if __name__ == '__main__':
    main()
