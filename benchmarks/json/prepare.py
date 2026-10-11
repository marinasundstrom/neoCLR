#!/usr/bin/env python3
"""Fresh-build JSON worker using a qualified macOS HTTP toolchain recipe."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    if not recipe['passed'] or not recipe.get('jitViaRuntimeCli'):
        raise ValueError('Passing CLI HTTP recipe required')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    old = Path(recipe['artifacts']['client']['commands']['aot'][0]).parent
    jit = recipe['artifacts']['client']['commands']['jit']
    old_host = jit[jit.index('--jit-host')+1]
    source = ROOT/'benchmarks/json/Nodes.rvn'
    worker = ROOT/'benchmarks/framework/Worker.rvn'
    host = out/'host.dylib'
    def replace(command):
        return [str(source) if part == recipe['artifacts']['client']['source'] else
                str(host) if part == old_host else str(out/Path(part).name)
                if Path(part).is_absolute() and Path(part).parent == old else part for part in command]
    (out/'main.c').write_text((old/'main.c').read_text())
    report = dict(passed=False, builds=[], inputs={})
    for path in [Path(__file__), worker, args.recipe, *args.compiler.resolve().parent.iterdir()]:
        if path.is_file() and (path.suffix in {'.dll', '.json', '.rvn', '.py'}):
            report['inputs'][str(path.resolve())] = sha(path)
    try:
        for entry in [recipe['builds'][0], *recipe['builds'][4:7]]:
            command = replace(entry['command'])
            if command[0] == 'dotnet':
                command[1] = str(args.compiler.resolve())
                command.append(str(worker))
            for part in command:
                if Path(part).is_absolute() and Path(part).is_file():
                    report['inputs'].setdefault(part, sha(part))
            start = time.perf_counter()
            result = subprocess.run(command, capture_output=True, text=True, timeout=300)
            report['builds'].append(dict(command=command, seconds=time.perf_counter()-start,
                exitCode=result.returncode, stdout=result.stdout if result.returncode else '',
                stdoutSha256=hashlib.sha256(result.stdout.encode()).hexdigest(), stderr=result.stderr))
            if result.returncode:
                raise RuntimeError(result.stdout+result.stderr)
        commands = {mode:replace(command) for mode,command in recipe['artifacts']['client']['commands'].items()}
        if '--execution' not in commands['interpreter']:
            commands['interpreter'] += ['--execution', 'interpreter']
        for command in commands.values():
            for part in command:
                if Path(part).is_absolute() and Path(part).is_file():
                    report['inputs'].setdefault(part,sha(part))
        manifest = dict(schemaVersion=1, commands=commands, inputs=report['inputs'],
                        cases=['json.nodes.serialize','json.nodes.deserialize'])
        (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        report['passed']=True
    finally:
        (out/'build.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':
    main()
