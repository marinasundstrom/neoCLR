#!/usr/bin/env python3
"""Prepare async PostgreSQL workers from a passing SQL query/AOT qualification recipe."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True)
    parser.add_argument('--postgres-port', type=int, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.postgres_port <= 65535:
        parser.error('Expected a valid loopback port')
    recipe = json.loads(args.recipe.read_text())
    if not recipe.get('passed'):
        raise ValueError('Passing SQL qualification required')
    for path, expected in recipe['inputs'].items():
        if hashlib.sha256(Path(path).read_bytes()).hexdigest() != expected:
            raise ValueError('Qualification input changed: ' + path)
    commands = {record['label']: record['command'] for record in recipe['commands']}
    for label in ('Main-compile', 'metadata', 'postgres-jit', 'postgres-interpreter', 'postgres-aot-compile', 'postgres-aot-link'):
        if label not in commands:
            raise ValueError('Recipe must qualify interpreter, JIT and AOT: missing ' + label)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    old = Path(commands['Main-compile'][commands['Main-compile'].index('-o') + 1]).parent
    source = Path(__file__).with_name('Worker.rvn').resolve()
    config = out / 'Configuration.rvn'
    config.write_text('internal static class Configuration {\n    static val Port: int => '
                      + str(args.postgres_port) + '\n}\n')
    (out / 'main.c').write_bytes((old / 'main.c').read_bytes())
    report = dict(passed=False, commands=[], inputs=dict(recipe['inputs']), postgresPort=args.postgres_port)

    def replace(command):
        return [str(source) if part == str(ROOT / 'docs/experiments/sql-postgresql/Main.rvn') else
                str(out / Path(part).name) if Path(part).is_absolute() and Path(part).parent == old else part
                for part in command]

    def run(label, command):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600)
        report['commands'].append(dict(label=label, command=command, exitCode=result.returncode,
                                       stdout=result.stdout if result.returncode else '', stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)

    try:
        run('compile', replace(commands['Main-compile']) + [str(config)])
        run('metadata', replace(commands['metadata']))
        listing = (out / 'Main.neoil').read_text()
        module, _ = json.JSONDecoder().raw_decode(listing[listing.index('{'):])
        roots = []
        for name in ('BenchmarkRow', 'QueryParameters'):
            suffix = '.T__' + name.encode().hex().upper()
            matches = [i for i, item in enumerate(module['types']) if item['name'].endswith(suffix)]
            if len(matches) != 1:
                raise ValueError('Expected one metadata type: ' + name)
            roots.append(dict(definition=dict(module=module['name'], revision=module['revision'], index=matches[0]),
                              construct=True, properties=True, getters=True, setters=True))
        (out / 'reflection.json').write_text(json.dumps(dict(schemaVersion=2, types=roots), indent=2) + '\n')
        run('aot', replace(commands['postgres-aot-compile']))
        link = replace(commands['postgres-aot-link'])
        link[0] = os.environ.get('CC', link[0])
        run('link', link)
        interpreter = replace(commands['postgres-interpreter'])
        interpreter = interpreter[:interpreter.index('--')]
        interpreter[interpreter.index('--instructions') + 1] = '2000000000'
        jit = list(interpreter)
        jit[jit.index('--execution') + 1] = 'jit'
        index = jit.index('--instructions')
        del jit[index:index + 2]
        native = commands['postgres-jit']
        host = native[native.index('@entry') + 1]
        jit += ['--jit-host', host, '--jit-compiler', native[0], '--reflection-roots', str(out / 'reflection.json')]
        paths = [Path(__file__).resolve(), source, config, args.recipe.resolve(), out / 'main.c',
                 out / 'Main.dll', out / 'Main.o', out / 'app', out / 'reflection.json']
        for path in paths:
            report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        for path, expected in report['inputs'].items():
            if hashlib.sha256(Path(path).read_bytes()).hexdigest() != expected:
                raise ValueError('Qualification input changed: ' + path)
        manifest = dict(schemaVersion=1, commands=dict(interpreter=interpreter, jit=jit, aot=[str(out / 'app')]),
                        cases=['postgres.positional-1', 'postgres.named-32'], inputs=report['inputs'])
        (out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        report['passed'] = True
    finally:
        (out / 'build.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
