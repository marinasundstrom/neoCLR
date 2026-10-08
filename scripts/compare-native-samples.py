#!/usr/bin/env python3
"""Inventory Raven samples through emission, interpreter, AOT, link and execution.

This is a bounded coverage survey, not an acceptance gate or a benchmark. Failures
are retained, not skipped. HTTP requires its separate peer-driving verifier.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'runtime', 'aot', 'bundle', 'output'):
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--case', action='append', help='Limit to named samples')
    a = p.parse_args()
    compiler, runtime, aot, bundle, out = (getattr(a, k).resolve() for k in
                                          ('compiler', 'runtime', 'aot', 'bundle', 'output'))
    out.mkdir(parents=True, exist_ok=False)
    base = ROOT / 'docs/experiments/aot-console'
    samples = {f.stem: [f] for f in sorted((ROOT / 'docs/experiments/raven-target/samples').glob('*.rvn'))}
    for name in ('native-async-entry-int', 'native-async-entry-cancelled', 'native-async-entry-pending',
                 'native-async-propagation', 'native-async-state'):
        samples[name] = [ROOT / 'docs/experiments/extended-cli-metadata/bootstrap' / (name + '.rvn')]
    samples['json-object-mapping'] = [ROOT / 'docs/experiments/json-object-mapping' / n for n in ('Mapping.rvn', 'Main.rvn')]
    if a.case:
        samples = {name: samples[name] for name in a.case}
    lib = bundle / 'lib' if (bundle / 'lib').is_dir() else bundle
    refs = [lib / n for n in ('System.Runtime.dll', 'System.Data.dll', 'System.Networking.dll', 'System.Web.dll')]
    seed, core, ownership = (lib / n for n in ('System.runtime.neox', 'Core.dll', 'ownership.json'))
    context = ['--system', seed, *[x for f in refs for x in ('--module', f)], '--object-root', refs[0]]
    flags = ['--compile-system', '--bind-user-fault', '--reference-arena', '--native-gc',
             '--native-stack-budget', '--bind-console-read-byte', '--bind-console-write-line',
             '--bind-console-stream-output', '--bind-int32-to-string', '--bind-utf8-text', '--bind-paths',
             '--bind-character-text', '--bind-integer-text', '--bind-task-queue',
             '--bind-socket-listener', '--bind-socket-accept', '--bind-socket-transfer']
    adapters = [base / n for n in ('root-probe.c', 'native-gc.c', 'native-stack.c', 'task-queue.c',
                                   'socket-listener.c', 'text-arena.c', 'console.c')]
    adapters += [base.parent / 'aot-scalar/console.c', base.parent / 'aot-fault-details/render.c']
    sha = lambda f: hashlib.sha256(f.read_bytes()).hexdigest()
    inputs = [Path(__file__), compiler, runtime, aot, core, seed, ownership, *refs, *adapters,
              ROOT / 'benchmarks/native-web/http-host.c', *base.glob('*.h'),
              *compiler.parent.glob('*.dll'), *ROOT.joinpath('tools/aot-poc/src').glob('*.rs')]
    report = dict(scope='104-case development-bundle survey; first blockers only, not full platform coverage or release qualification',
                  revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  dirty=subprocess.check_output(['git', 'status', '--short'], cwd=ROOT, text=True),
                  host=platform.platform(), SDKROOT=os.environ.get('SDKROOT'),
                  inputs={str(f): sha(f) for f in inputs}, cases=[])
    def save():
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

    def run(command, cwd, timeout=45, data=b'AB'):
        start = time.monotonic()
        command = list(map(str, command))
        try:
            r = subprocess.run(command, cwd=cwd, input=data, capture_output=True, timeout=timeout,
                               env={**os.environ, 'NEOCLR_API_PRESENT': 'Present', 'NEOCLR_API_EMPTY': ''})
            stdout = r.stdout.decode(errors='replace')
            return dict(command=command, exit=r.returncode, stdout=stdout if len(stdout) < 16000 else
                        dict(bytes=len(r.stdout), sha256=hashlib.sha256(r.stdout).hexdigest()),
                        stderr=r.stderr.decode(errors='replace'), seconds=time.monotonic()-start)
        except subprocess.TimeoutExpired as e:
            return dict(command=command, timeoutSeconds=timeout, stdout=(e.stdout or b'').decode(errors='replace'),
                        stderr=(e.stderr or b'').decode(errors='replace'), seconds=time.monotonic()-start)

    # Reuse the HTTP lifecycle host, conditionally omitting adapters not exported
    # by this particular closed world. The sample's CIL remains unchanged.
    host_source = (ROOT / 'benchmarks/native-web/http-host.c').read_text()
    host_source = host_source.replace('status = neoclr_drain_default_queue_v1(&context);',
        '#ifdef HAS_QUEUE\n        status = neoclr_drain_default_queue_v1(&context);\n#endif')
    host_source = host_source.replace('if (ready == 1) status = neoclr_invoke_void_callback_v1(callback, &context);',
        'if (ready == 1) {\n#ifdef HAS_CALLBACK\n status = neoclr_invoke_void_callback_v1(callback, &context);\n#else\n host_error = 1;\n#endif\n}')
    host = out / 'host.c'
    host.write_text(host_source)
    report['hostSha256'] = sha(host)
    for name, sources in samples.items():
        directory = out / name
        directory.mkdir()
        assembly, obj, binary = (directory / n for n in ('sample.dll', 'sample.o', 'sample'))
        case = dict(name=name, sources={str(f.relative_to(ROOT)): sha(f) for f in sources})
        report['cases'].append(case)
        compile_cmd = ['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
                       '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--object-library', 'System.Runtime',
                       '--async-library', 'System.Runtime', *[x for f in refs for x in ('--reference', f)], '-o', assembly, *sources]
        case['emission'] = run(compile_cmd, directory, 120)
        if case['emission'].get('exit') != 0:
            case['status'] = 'emission-blocked'
        else:
            case['assemblySha256'] = sha(assembly)
            # Separate working directories prevent one backend's file writes
            # from becoming another backend's inputs.
            vm_dir, native_dir = directory / 'vm-work', directory / 'native-work'
            vm_dir.mkdir(); native_dir.mkdir()
            guest_args = ['--', 'argument'] if name in ('library-environment', 'native-async-entry-int') else []
            case['interpreter'] = run([runtime, 'run', assembly, *context, '--instructions', '100000000', *guest_args], vm_dir)
            case['aot'] = run([aot, '--closed-world', assembly, '@entry', obj, *context, *flags], directory, 120)
            if case['aot'].get('exit') != 0:
                case['status'] = 'aot-blocked'
            else:
                symbols = subprocess.check_output(['nm', '-g', obj], text=True)
                defines = [f'-D{macro}' for symbol, macro in (('_neoclr_drain_default_queue_v1', 'HAS_QUEUE'),
                           ('_neoclr_invoke_void_callback_v1', 'HAS_CALLBACK')) if symbol in symbols]
                case['link'] = run(['clang', '-arch', 'arm64', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                                    '-DNEOCLR_NATIVE_GC', *defines, '-I', base, host, *adapters, obj, '-o', binary], directory)
                if case['link'].get('exit') != 0:
                    case['status'] = 'link-blocked'
                else:
                    case['native'] = run([binary], native_dir)
                    case['dependencies'] = run(['otool', '-L', binary], directory)
                    vm, native = case['interpreter'], case['native']
                    case['sameExitStdout'] = (vm.get('exit') is not None and vm.get('exit') == native.get('exit') and vm['stdout'] == native['stdout'])
                    case['status'] = 'matched-success' if case['sameExitStdout'] and vm['exit'] == 0 and not vm['stderr'] and not native['stderr'] else 'execution-review'
        save()
        print(f'{name}: {case["status"]}', flush=True)
    # Inputs changing mid-survey invalidate a reproducible comparison.
    report['inputsUnchanged'] = all(sha(Path(f)) == digest for f, digest in report['inputs'].items())
    report['summary'] = {status: sum(c['status'] == status for c in report['cases']) for status in sorted({c['status'] for c in report['cases']})}
    save()
    print(json.dumps(report['summary']))


if __name__ == '__main__':
    main()
