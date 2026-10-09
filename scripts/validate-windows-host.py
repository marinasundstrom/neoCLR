#!/usr/bin/env python3
"""Qualify private Windows host memory prerequisites, and bounded integer AOT stack codegen."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, scope='Private Windows x64 guarded heap and collector/root lifecycle; bounded native stack probe; integer-only generated stack experiment; no managed service qualification',
                  platform=platform.platform(), commands=[])

    def run(command, name):
        command = list(map(str, command))
        result = subprocess.run(command, cwd=out, capture_output=True, timeout=360)
        (out / (name + '.stdout.log')).write_bytes(result.stdout)
        (out / (name + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(command=command, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(f'{name} failed: see retained logs')
        return result

    try:
        if platform.system() != 'Windows' or platform.machine().lower() not in ('amd64', 'x86_64'):
            raise ValueError('Requires Windows x64 and an MSVC developer shell')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        inputs = [ROOT / 'tools/native' / name for name in
                  ('windows-host-memory.c', 'windows-host-memory.h', 'windows-host-memory-test.c')]
        report['inputs'] = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/O2', '/MT', '/Fe:host-memory.exe', inputs[0], inputs[2]], 'build')
        result = run([out / 'host-memory.exe'], 'execute')
        expected = b'Windows guarded heap: 12 allocation lifecycles passed\r\n'
        if result.stdout != expected or result.stderr:
            raise ValueError('Host memory acceptance did not report exact completion')
        report['execution'] = dict(exitCode=0, stdout=result.stdout.decode(), stderr='', allocationLifecycles=12)
        base = ROOT / 'docs/experiments/aot-console'
        collector_inputs = [ROOT / 'tools/native' / name for name in
                            ('windows-gc-host.c', 'windows-gc-host.h', 'windows-gc-host-test.c', 'windows-native-stack.c')]
        collector_inputs += [base / name for name in ('native-gc.c', 'native-gc-test.c', 'root-probe.c', 'native-gc.h', 'root-probe.h', 'text-arena.h', 'native-stack.h')]
        collector_inputs += [ROOT / 'docs/experiments/aot-fault-details/fault-details.h']
        report['inputs'].update({p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in collector_inputs})
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
             '/Fe:host-collector.exe', ROOT / 'tools/native/windows-gc-host-test.c',
             ROOT / 'tools/native/windows-gc-host.c', ROOT / 'tools/native/windows-native-stack.c',
             inputs[0], base / 'native-gc.c', base / 'root-probe.c'], 'collector-build')
        result = run([out / 'host-collector.exe'], 'collector-execute')
        expected = b'Windows collector: rooted graph, frame handoff, thread isolation, exhaustion and cleanup passed\r\n'
        if result.stdout != expected or result.stderr:
            raise ValueError('Collector acceptance did not report exact completion')
        report['collectorExecution'] = dict(exitCode=0, stdout=result.stdout.decode(), stderr='')
        # Existing collector semantics, independently of the new host wrapper.
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
             '/Fe:collector-contract.exe', base / 'native-gc-test.c', base / 'native-gc.c', base / 'root-probe.c'], 'collector-contract-build')
        result = run([out / 'collector-contract.exe'], 'collector-contract-execute')
        if result.stdout or result.stderr:
            raise ValueError('Unexpected collector contract output')
        report['collectorContract'] = dict(exitCode=0)
        stack_inputs = [ROOT / 'tools/native' / name for name in
                        ('windows-native-stack.c', 'windows-native-stack-test.c')]
        stack_inputs += [base / 'native-stack.h']
        report['inputs'].update({p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in stack_inputs})
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
             '/Fe:host-stack.exe', *stack_inputs[:2], ROOT / 'tools/native/windows-gc-host.c',
             inputs[0], base / 'native-gc.c', base / 'root-probe.c'], 'stack-build')
        result = run([out / 'host-stack.exe'], 'stack-execute')
        stack = json.loads(result.stdout)
        if (result.stderr or stack.get('passed') is not True
                or stack.get('smallStackRejected') is not True or stack.get('fiberRejected') is not True
                or not 1 < stack.get('depth512KiB', 0) < stack.get('depth1MiB', 0) < 64):
            raise ValueError('Stack acceptance did not complete boundary and return checks')
        report['stackExecution'] = stack
        generated_inputs = [ROOT / 'tools/native' / name for name in
                            ('windows-generated-stack.neoil', 'windows-generated-stack-test.c')]
        generated_inputs += list((ROOT / 'tools/aot-poc/src').glob('*.rs'))
        generated_inputs += [ROOT / 'tools/aot-poc/Cargo.toml', ROOT / 'tools/aot-poc/Cargo.lock']
        report['inputs'].update({p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in generated_inputs})
        (out / generated_inputs[0].name).write_bytes(generated_inputs[0].read_bytes())
        manifest = ROOT / 'tools/aot-poc/Cargo.toml'
        run(['rustc', '--version', '--verbose'], 'rust-toolchain')
        run(['cargo', 'test', '--locked', '--manifest-path', manifest, '--test', 'windows_stack'], 'generated-contract')
        run(['cargo', 'test', '--locked', '--manifest-path', manifest, '--bin', 'neoclr-aot-poc', 'windows_final_frame_limit'], 'generated-frame-boundary')
        compiler = ROOT / 'tools/aot-poc/target/debug/neoclr-aot-poc.exe'
        report['compilerSha256'] = hashlib.sha256(compiler.read_bytes()).hexdigest()
        run([compiler, generated_inputs[0], 'Calculate', out / 'generated-stack.obj',
             '--target', 'x86_64-pc-windows-msvc', '--windows-stack-experiment',
             '--reference-arena', '--native-gc', '--native-stack-budget'], 'generated-compile')
        run(['dumpbin', '/disasm', out / 'generated-stack.obj'], 'generated-disassembly')
        run(['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT',
             '/DNEOCLR_NATIVE_GC', '/Fe:generated-stack.exe', generated_inputs[1],
             ROOT / 'tools/native/windows-gc-host.c', ROOT / 'tools/native/windows-native-stack.c',
             inputs[0], base / 'native-gc.c', base / 'root-probe.c', out / 'generated-stack.obj'], 'generated-build')
        result = run([out / 'generated-stack.exe'], 'generated-execute')
        generated = json.loads(result.stdout)
        if (result.stderr or generated.get('passed') is not True
                or generated.get('smallStackRejected') is not True or generated.get('reusePassed') is not True
                or not 5 < generated.get('snapshots512KiB', 0) < generated.get('snapshots1MiB', 0) < 512):
            raise ValueError('Generated stack acceptance did not complete fault, cleanup and reuse checks')
        report['generatedStackExecution'] = generated
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Windows host memory: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
