#!/usr/bin/env python3
"""Qualify generic async and retained use scopes with both native Raven lowerings."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--compiler', type=Path, required=True)
    p.add_argument('--backend', type=Path, default=ROOT / 'tools/aot-poc/target/release/neoclr-aot-poc')
    p.add_argument('--recipe', type=Path, default=HERE / 'collector-walk-showcase.json')
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise RuntimeError('Runtime suspension qualification currently requires macOS ARM64')
    bundle, compiler, backend, out = [x.resolve() for x in (a.bundle, a.compiler, a.backend, a.output)]
    out.mkdir(parents=True, exist_ok=False)
    recipe = json.loads(a.recipe.read_text())
    if not recipe['passed']:
        raise ValueError('Requires a passing host recipe')
    commands = {row['label']: row['command'] for row in recipe['commands']}
    source = commands['use-source']
    old_lib = str(Path(source[source.index('--core-reference') + 1]).parent)
    replace = lambda c: [x.replace(old_lib, str(bundle / 'lib')) for x in c]
    source = replace(source[:source.index('-o')])
    source[1] = str(compiler)
    native = replace(commands['use-native-compile'])
    native[0] = str(backend)
    link = commands['use-link']
    old_obj, old_app = native[4], link[link.index('-o') + 1]
    interpreter = replace(commands['contract-interpreter'])
    interpreter[0] = str(bundle / 'bin/neoclr')
    interpreter += ['--execution', 'interpreter']
    manifest = json.loads((bundle / 'manifest.json').read_text())
    for relative, digest in manifest['files'].items():
        path = (bundle / relative).resolve()
        if bundle not in path.parents or sha(path) != digest:
            raise ValueError('Bundle file mismatch: ' + relative)
    inputs = {str(a.recipe.resolve()): sha(a.recipe), str(Path(__file__).resolve()): sha(__file__),
              str(backend): sha(backend), str(bundle / 'manifest.json'): sha(bundle / 'manifest.json')}
    for path in compiler.parent.glob('*.dll'):
        inputs[str(path)] = sha(path)
    for argument in link:
        path = Path(argument)
        if path.is_file() and path.suffix in ('.c', '.a'):
            inputs[str(path)] = sha(path)
    for directory in (HERE, ROOT / 'docs/experiments/aot-console', ROOT / 'tools/native'):
        for path in directory.glob('*.h'):
            inputs[str(path)] = sha(path)
    report = dict(passed=False, platform=platform.platform(), inputsSha256=inputs, commands=[], assemblies={})

    def run(command, label, expected=None, repetitions=1):
        command = list(map(str, command))
        result = subprocess.run(command, capture_output=True, text=True, timeout=300)
        report['commands'].append(dict(label=label, command=command, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode or (expected is not None and (result.stdout != (expected + '\n') * repetitions or result.stderr)):
            raise RuntimeError(label + ': ' + result.stdout + result.stderr)
        print('PASS ' + label, flush=True)

    try:
        for name, filename, expected in [
            ('use', 'RuntimeUse.rvn', 'PASS runtime async scoped cleanup'),
            ('generic-use', 'RuntimeGenericUse.rvn', 'PASS generic async methods owners and scoped cleanup'),
        ]:
            fixture = HERE / filename
            inputs[str(fixture)] = sha(fixture)
            inputs[str(HERE / 'RuntimeAsyncBridge.rvn')] = sha(HERE / 'RuntimeAsyncBridge.rvn')
            for lowering in ('runtime', 'state-machine'):
                key = name + '-' + lowering
                assembly, obj, app = [out / (key + suffix) for suffix in ('.dll', '.o', '')]
                compile_source = source.copy()
                if lowering == 'state-machine':
                    index = compile_source.index('--runtime-async-poc')
                    del compile_source[index:index + 2]
                compile_source += ['-o', assembly, fixture]
                if lowering == 'runtime':
                    compile_source += [HERE / 'RuntimeAsyncBridge.rvn']
                run(compile_source, key + '-source')
                report['assemblies'][key] = sha(assembly)
                interpreted = interpreter.copy()
                interpreted[2] = str(assembly)
                run(interpreted, key + '-interpreter', expected)
                run([bundle / 'bin/neoclr', 'run', assembly, '--jit-compiler', backend], key + '-jit', expected)
                compile_native = native.copy()
                compile_native[2], compile_native[4] = str(assembly), str(obj)
                run(compile_native, key + '-native')
                app_link = [str(obj) if x == old_obj else str(app) if x == old_app else x for x in link]
                run(app_link, key + '-link')
                run([app], key + '-aot', expected)
                repeated = out / (key + '-repeat')
                run([str(repeated) if x == str(app) else x for x in app_link] + ['-DNEOCLR_HTTP_INVOCATIONS=4'], key + '-repeat-link')
                run([repeated], key + '-same-heap', expected, 4)
        if any(sha(path) != digest for path, digest in inputs.items()):
            raise RuntimeError('An input changed during qualification')
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
