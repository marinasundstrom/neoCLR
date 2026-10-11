#!/usr/bin/env python3
"""Build the async scheduler worker from a qualified macOS scheduler recipe."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    if not recipe['passed'] or recipe['cases'][0]['name'] != 'scheduler':
        raise ValueError('Passing scheduler-projection recipe required')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    commands = [record['command'] for record in recipe['commands'][:5]]
    host, compile_command, native, link, interpret = commands
    old_host = host[host.index('-o')+1]
    old_dir = Path(compile_command[compile_command.index('-o')+1]).parent
    source = Path(__file__).with_name('StateMachine.rvn').resolve()
    lib = Path(compile_command[compile_command.index('--core-reference')+1]).parent

    def replace(command):
        result = []
        for part in command:
            path = Path(part)
            if part == compile_command[-1]:
                part = str(source)
            elif part == old_host:
                part = str(out/'host.dylib')
            elif path.is_absolute() and path.parent == old_dir:
                part = str(out/path.name)
            elif path.is_absolute() and path.name in {
                    'System.runtime.neox', 'System.Runtime.dll', 'System.Data.dll',
                    'System.Networking.dll', 'System.Web.dll'}:
                part = str(lib/path.name)
            result.append(part)
        return result

    # The benchmark has an argument-taking async entry in every execution mode.
    (out/'main.c').write_text(
        '#include "native-gc.h"\n'
        'extern int32_t neoclr_entry_args_v1(int32_t, int32_t *, neoclr_aot_context *, const char **);\n'
        'extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);\n'
        'int main(int argc, char **argv) { return neoclr_jit_run_v1((const void *)'
        'neoclr_entry_args_v1, 1, argc, (const char **)argv); }\n')
    report = dict(passed=False, sourceBaseRevision=subprocess.check_output(
        ['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(), builds=[], inputs={})
    provenance = [Path(__file__).resolve(), source, args.recipe.resolve(), out/'main.c']
    provenance += list(Path(compile_command[1]).parent.glob('*.dll'))
    provenance += list(Path(compile_command[1]).parent.glob('*.json'))
    provenance += list((ROOT/'docs/experiments/aot-console').glob('*.h'))
    for path in provenance:
        report['inputs'][str(path)] = sha(path)
    try:
        for command in commands[:4]:
            command = replace(command)
            for part in command:
                path = Path(part)
                if path.is_absolute() and path.is_file():
                    report['inputs'][part] = sha(path)
            result = subprocess.run(command, capture_output=True, text=True, timeout=300)
            report['builds'].append(dict(command=command, exitCode=result.returncode,
                stdout=result.stdout if result.returncode else '', stderr=result.stderr))
            if result.returncode:
                raise RuntimeError(result.stdout+result.stderr)
        interpret = replace(interpret) + ['--execution', 'interpreter']
        jit = replace(interpret[:-2]) + ['--execution', 'jit', '--jit-host', str(out/'host.dylib'),
               '--jit-compiler', native[0]]
        limit = jit.index('--instructions')
        del jit[limit:limit+2]
        modes = dict(interpreter=interpret, jit=jit, aot=[str(out/'app')])
        for command in modes.values():
            for part in command:
                if Path(part).is_absolute() and Path(part).is_file():
                    report['inputs'][part] = sha(part)
        manifest = dict(schemaVersion=1, commands=modes, inputs=report['inputs'],
                        cases=['await.completed', 'await.pending'])
        (out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
        report['passed'] = True
    finally:
        (out/'build.json').write_text(json.dumps(report, indent=2)+'\n')


if __name__ == '__main__':
    main()
