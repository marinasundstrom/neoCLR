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
    parser.add_argument('--source', type=Path, help='Alternative worker source')
    parser.add_argument('--discover', action='store_true', help='Discover BenchmarkAttribute module functions before final compilation')
    parser.add_argument('--manual-case', action='append', default=[], help='Explicit manual selector to include alongside discovered functions')
    args = parser.parse_args()
    if args.manual_case and not args.discover:
        parser.error('--manual-case requires --discover')
    if not args.source and not args.discover:
        parser.error('The JSON node worker requires --discover --manual-case json.nodes.serialize')
    recipe = json.loads(args.recipe.read_text())
    if not recipe['passed'] or not recipe.get('jitViaRuntimeCli'):
        raise ValueError('Passing CLI HTTP recipe required')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    old = Path(recipe['artifacts']['client']['commands']['aot'][0]).parent
    jit = recipe['artifacts']['client']['commands']['jit']
    old_host = jit[jit.index('--jit-host')+1]
    source = args.source.resolve() if args.source else ROOT/'benchmarks/json/Nodes.rvn'
    worker = ROOT/'benchmarks/framework/Worker.rvn'
    host = out/'host.dylib'
    def replace(command):
        return [str(source) if part == recipe['artifacts']['client']['source'] else
                str(host) if part == old_host else str(out/Path(part).name)
                if Path(part).is_absolute() and Path(part).parent == old else part for part in command]
    (out/'main.c').write_text((old/'main.c').read_text())
    report = dict(passed=False, builds=[], inputs={})
    registry = out/'BenchmarkRegistry.rvn'
    discovery = out/'benchmarks.json'
    attribute = ROOT/'benchmarks/framework/BenchmarkAttribute.rvn'
    if args.discover:
        registry.write_text('module NeoClr.Benchmarking\nfunc RegisterDiscoveredBenchmarks(suite: BenchmarkSuite) { }\n')
    def discover(verified=False):
        target_registry = out/'VerifiedRegistry.rvn' if verified else registry
        target_manifest = out/'verified-benchmarks.json' if verified else discovery
        dependencies = [entry['command'][i+1] for entry in recipe['builds'][4:5]
                        for i, part in enumerate(entry['command']) if part == '--reference']
        project = ROOT/'tools/benchmarking/NeoCLR.BenchmarkDiscovery'
        command = ['dotnet','run','--project',str(project),'-p:WarningLevel=0','--',
                   str(out/'app.dll'),str(target_registry),str(target_manifest),*dependencies]
        result = subprocess.run(command,capture_output=True,text=True,timeout=300)
        report['builds'].append(dict(command=command,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stderr)
        if verified and (target_registry.read_bytes()!=registry.read_bytes() or target_manifest.read_bytes()!=discovery.read_bytes()):
            raise RuntimeError('Final assembly changed benchmark discovery')
        return json.loads(target_manifest.read_text())

    for path in [Path(__file__), worker, args.recipe, *args.compiler.resolve().parent.iterdir()]:
        if path.is_file() and (path.suffix in {'.dll', '.json', '.rvn', '.py'}):
            report['inputs'][str(path.resolve())] = sha(path)
    try:
        for entry in [recipe['builds'][0], *recipe['builds'][4:7]]:
            command = replace(entry['command'])
            if command[0] == 'dotnet':
                command[1] = str(args.compiler.resolve())
                command.append(str(worker))
                if args.discover:
                    command += [str(attribute),str(registry)]
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
            if command[0] == 'dotnet' and args.discover:
                discover()
                (out/'app.dll').unlink()  # Replace only this run's inspection assembly.
                result = subprocess.run(command,capture_output=True,text=True,timeout=300)
                report['builds'].append(dict(command=command,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
                if result.returncode:
                    raise RuntimeError(result.stdout+result.stderr)
                discovered = discover(verified=True)
                for path in [registry,discovery,attribute,* (ROOT/'tools/benchmarking/NeoCLR.BenchmarkDiscovery').glob('*.cs*')]:
                    report['inputs'][str(path)] = sha(path)

        commands = {mode:replace(command) for mode,command in recipe['artifacts']['client']['commands'].items()}
        if '--execution' not in commands['interpreter']:
            commands['interpreter'] += ['--execution', 'interpreter']
        for command in commands.values():
            for part in command:
                if Path(part).is_absolute() and Path(part).is_file():
                    report['inputs'].setdefault(part,sha(part))
        cases = [item['name'] for item in discovered['benchmarks']] + args.manual_case if args.discover else ['json.nodes.serialize','json.nodes.deserialize']
        if any(not name for name in cases) or len(cases) != len(set(cases)):
            raise ValueError('Empty or duplicate benchmark selector')
        manifest = dict(schemaVersion=1, commands=commands, inputs=report['inputs'], cases=cases)
        (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        report['passed']=True
    except Exception:
        if args.discover:
            for path in [registry, discovery, out/'VerifiedRegistry.rvn', out/'verified-benchmarks.json', out/'manifest.json']:
                path.unlink(missing_ok=True)
        raise
    finally:
        (out/'build.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':
    main()
