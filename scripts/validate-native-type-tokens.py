#!/usr/bin/env python3
"""Execute the private native type-token foundation on macOS ARM64 and Windows x64."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--aot', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime-equality', action='store_true', help='Exercise the TypeEquals runtime service instead of direct token equality')
    parser.add_argument('--descriptor-queries', action='store_true', help='Exercise semantic names and generic arity')
    parser.add_argument('--primitive-boxes', action='store_true', help='Exercise scalar box identity and checked unboxing')
    args = parser.parse_args()
    aot, out = args.aot.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), machine=platform.machine(), commands=[], runtimeEquality=args.runtime_equality, descriptorQueries=args.descriptor_queries, primitiveBoxes=args.primitive_boxes)
    def run(command, name):
        r = subprocess.run(list(map(str, command)), cwd=out, capture_output=True, timeout=120)
        (out / (name + '.stdout')).write_bytes(r.stdout)
        (out / (name + '.stderr')).write_bytes(r.stderr)
        report['commands'].append(dict(command=r.args, exitCode=r.returncode))
        if r.returncode:
            raise RuntimeError(name + ' failed; see logs')
        return r
    try:
        windows = platform.system() == 'Windows'
        if not windows and platform.system() != 'Darwin':
            raise ValueError('Requires macOS ARM64 or Windows x64')
        report['revision'] = run(['git', '-C', ROOT, 'rev-parse', 'HEAD'], 'revision').stdout.decode().strip()
        base = ROOT / 'docs/experiments/aot-console'
        sources = [base / n for n in ('type-tokens-test.c', 'native-gc.c', 'root-probe.c', 'text-arena.c')]
        inputs = [*sources, base / 'type-tokens.neoil', *[base / n for n in ('native-gc.h', 'root-probe.h', 'text-arena.h', 'native-stack.h')],
                  base.parent / 'aot-fault-details/fault-details.h', Path(__file__).resolve()]
        if args.descriptor_queries:
            sources[0] = base / 'type-descriptors-test.c'
            inputs += [sources[0], base / 'type-descriptors.neoil']
        if args.primitive_boxes:
            sources[0] = base / 'primitive-boxes-test.c'
            inputs += [sources[0], base / 'primitive-boxes.neoil']
        if windows:
            sources += [ROOT / 'tools/native/windows-native-stack.c']
            inputs += [sources[-1]]
            cc = ['cl', '/nologo', '/W4', '/WX', '/std:c11', '/experimental:c11atomics', '/O2', '/MT', '/DNEOCLR_NATIVE_GC', '/I' + str(base)]
        else:
            clang = run(['xcrun', '--find', 'clang'], 'clang').stdout.decode().strip()
            sdk = run(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], 'sdk').stdout.decode().strip()
            cc = [clang, '-isysroot', sdk, '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-I', base, '-fsanitize=undefined,bounds']
        report['inputs'] = {p.relative_to(ROOT).as_posix(): sha(p) for p in inputs}
        report['aotSha256'] = sha(aot)
        seed, helper = out / 'System.neoil', out / 'Helpers.neoil'
        seed.write_text('.module System\n.references ()\n' + ('.function neoCLR.Runtime.TypeEquals(RuntimeTypeHandle,RuntimeTypeHandle) -> Boolean\n.methodimpl InternalCall\n.end\n' if args.runtime_equality or args.descriptor_queries else ''))
        if args.descriptor_queries:
            with seed.open('a') as f:
                f.write('.type System.Int32\n.end\n.function neoCLR.Runtime.TypeName(RuntimeTypeHandle) -> String\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeArgumentCount(RuntimeTypeHandle) -> Int32\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.TypeShape(RuntimeTypeHandle,Int32) -> Boolean\n.methodimpl InternalCall\n.end\n.function neoCLR.Runtime.WriteLine(String) -> Void\n.methodimpl InternalCall\n.end\n')
        if args.primitive_boxes:
            seed.write_text('.module System\n.references ()\n')
        source = base / 'type-tokens.neoil'
        if args.runtime_equality:
            source = out / 'type-equality.neoil'
            source.write_text((base / 'type-tokens.neoil').read_text().replace('ceq', 'call neoCLR.Runtime.TypeEquals(RuntimeTypeHandle,RuntimeTypeHandle)'))
        helper.write_text('.module Helpers\n.references ()\n')
        if args.descriptor_queries:
            source = base / 'type-descriptors.neoil'
        if args.primitive_boxes:
            source = base / 'primitive-boxes.neoil'
        obj = out / ('guest.obj' if windows else 'guest.o')
        flags = ['--compile-system', '--reference-arena', '--native-gc']
        if args.descriptor_queries:
            flags += ['--bind-console-write-line']
        if windows:
            flags += ['--target', 'x86_64-pc-windows-msvc', '--windows-console-experiment', '--native-stack-budget']
        run([aot, '--closed-world', source, 'Calculate', obj, '--system', seed, '--module', helper, *flags], 'guest-build')
        exe = out / ('host.exe' if windows else 'host')
        link = ['/Fe:' + str(exe), '/link', '/STACK:1048576'] if windows else ['-o', exe]
        run([*cc, *sources, obj, *link], 'host-build')
        r = run([exe], 'execute')
        expected = (b'System.Int32\nAccount\nModel\nModel\n' if args.descriptor_queries else b'') + b'Type tokens: 42\n'
        if args.primitive_boxes:
            expected = b'Primitive boxes: 42\n'
        if r.stdout.replace(b'\r\n', b'\n') != expected or r.stderr:
            raise ValueError('Unexpected token identity result')
        if sha(aot) != report['aotSha256'] or any(sha(ROOT / n) != h for n, h in report['inputs'].items()):
            raise ValueError('Inputs changed during validation')
        report.update(passed=True, stdout=expected.decode(), stderr='')
    except Exception as error:
        report['error'] = str(error)
    finally:
        report['files'] = {p.name: sha(p) for p in sorted(out.iterdir()) if p.is_file()}
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Native type tokens: ' + ('PASS' if report['passed'] else 'FAIL: ' + report['error']))
    return 0 if report['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
