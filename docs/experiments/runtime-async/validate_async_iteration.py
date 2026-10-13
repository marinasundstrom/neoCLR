#!/usr/bin/env python3
"""Qualify the managed async iteration protocol in all three POC modes."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True, help='Passing validate_native.py report')
    parser.add_argument('--interpreter-recipe', type=Path, required=True, help='Passing source/interpreter report')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--libraries', type=Path, required=True)
    scenarios = parser.add_mutually_exclusive_group()
    scenarios.add_argument('--await-for', action='store_true', help='Qualify retained-body await-for consumption')
    scenarios.add_argument('--synchronous-for', action='store_true', help='Recheck existing retained-body synchronous iteration')
    scenarios.add_argument('--generator', action='store_true', help='Qualify the source-local yield/resume prototype')
    scenarios.add_argument('--generator-language', action='store_true', help='Qualify native Raven yield syntax')
    parser.add_argument('--compiler', type=Path, help='Compiler to qualify instead of the recipe compiler')
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    interpreted = json.loads(args.interpreter_recipe.read_text())
    if not recipe['passed'] or not interpreted['passed']:
        raise RuntimeError('Command recipes must have passed')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, commands=[], cases=[], inputsSha256={
        str(args.recipe.resolve()): sha(args.recipe),
        str(args.interpreter_recipe.resolve()): sha(args.interpreter_recipe),
        str(Path(__file__).resolve()): sha(__file__),
    })

    def command(label):
        return next(row['command'].copy() for row in recipe['commands'] if row.get('label') == label)

    def run(label, cmd):
        cmd = list(map(str, cmd))
        for index, item in enumerate(cmd):
            path = Path(item)
            if path.name in ('Core.dll', 'System.runtime.neox', 'System.Runtime.dll', 'System.Data.dll',
                             'System.Networking.dll', 'System.Web.dll', 'ownership.json'):
                cmd[index] = str(args.libraries.resolve() / path.name)
        for item in cmd:
            path = Path(item)
            if path.is_file() and not path.is_relative_to(out):
                report['inputsSha256'][str(path.resolve())] = sha(path)
        result = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=120)
        row = dict(label=label, command=cmd, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        report['commands'].append(row)
        if result.returncode:
            raise RuntimeError(f'{label}: {result.stdout}\n{result.stderr}')
        if label.endswith('-compile'):
            artifact = out / (label + '.json')
            artifact.write_text(row.pop('stdout'))
            row.update(stdoutFile=str(artifact), stdoutSha256=sha(artifact))
        return row

    try:
        compiler = args.compiler.resolve() if args.compiler else Path(command('stack-reuse-source')[1])
        for path in compiler.parent.glob('*.dll'):
            report['inputsSha256'][str(path.resolve())] = sha(path)
        for folder in (HERE, ROOT / 'docs/experiments/aot-console', ROOT / 'docs/experiments/aot-fault-details'):
            for path in folder.glob('*.h'):
                report['inputsSha256'][str(path.resolve())] = sha(path)
        host = out / 'host.dylib'
        host_cmd = command('jit-host-build')
        host_cmd[host_cmd.index('-o') + 1] = str(host)
        run('host', host_cmd)
        report['hostSha256'] = sha(host)
        samples = [('contract', 'RuntimeAsyncIteration.rvn', 'PASS managed async iteration contract')]
        if args.await_for:
            samples = [('await-for', 'RuntimeAwaitFor.rvn', 'PASS await for async lifetime')]
        if args.synchronous_for:
            samples = [('for', 'RuntimeForAwait.rvn', 'PASS await in for state and lifetime')]
        if args.generator:
            samples = [('generator', 'RuntimeAsyncGenerator.rvn', 'PASS demand driven async generator prototype')]
        if args.generator_language:
            samples = [('generator-language', 'RuntimeGeneratorLanguage.rvn', 'PASS native Raven async generator language')]
        for lowering in (('runtime',) if args.await_for or args.synchronous_for or args.generator or args.generator_language else ('state-machine', 'runtime')):
            for name, filename, expected in samples:
                label = lowering + '-' + name
                assembly, obj, app = [out / (label + suffix) for suffix in ('.dll', '.o', '')]
                source = command('stack-reuse-source')
                source[1] = str(compiler)
                source[source.index('-o') + 1:] = [str(assembly), str(HERE / filename), str(ROOT / 'runtime/raven/compiler-services/RuntimeAwait.rvn')]
                if args.generator_language:
                    source.append(str(ROOT / 'runtime/raven/compiler-services/RuntimeGenerator.rvn'))
                if lowering == 'state-machine':
                    i = source.index('--runtime-async-poc')
                    del source[i:i + 2]
                run(label + '-source', source)
                listing = out / (label + '.neoil')
                run(label + '-listing', [interpreted['commands'][1]['command'][0], 'disassemble', assembly, listing])
                has_state_machine = 'AsyncStateMachine' in listing.read_text()
                if has_state_machine != (lowering == 'state-machine'):
                    raise RuntimeError('Unexpected lowering in ' + label)
                native = command('stack-reuse-native-compile')
                old_obj = native[4]
                native[2], native[4] = str(assembly), str(obj)
                run(label + '-compile', native)
                link = command('stack-reuse-link')
                link[link.index(old_obj)] = str(obj)
                link[link.index('-o') + 1] = str(app)
                run(label + '-link', link)
                interpreter = interpreted['commands'][1]['command'].copy()
                interpreter[2] = str(assembly)
                jit = [native[0], '--jit-run', *native[1:]]
                jit[5] = str(host)
                for mode, invocation in [('interpreter', interpreter), ('jit', jit), ('aot', [app])]:
                    row = run(label + '-' + mode, invocation)
                    good = row['stdout'].strip() == expected and not row['stderr']
                    report['cases'].append(dict(lowering=lowering, sample=name, mode=mode, passed=good,
                                               assemblySha256=sha(assembly)))
                    if not good:
                        raise RuntimeError('Unexpected output: ' + str(row))
        report['passed'] = True
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(passed=True, cases=len(report['cases']), report=str(out / 'validation.json'))))


if __name__ == '__main__':
    main()
