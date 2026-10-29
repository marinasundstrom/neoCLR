#!/usr/bin/env python3
"""Reject unsupported benchmark signatures and verify metadata-only discovery."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = json.loads(args.build.read_text())
    compile_template = next(x['command'] for x in report['builds'] if 'neoclr' in x['command'])
    discover_template = next(x['command'] for x in report['builds'] if any('BenchmarkDiscovery' in p for p in x['command']))
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    evidence = dict(passed=False, cases=[])
    cases = [
        ('row-arity', '[Benchmark]\n[Arguments(5)]\nfunc Invalid() -> int => 1', 'wrong arity'),
        ('row-type', '[Benchmark]\n[Arguments("five")]\nfunc Invalid(value: int) -> int => 1', 'wrong parameter type'),
        ('missing-source', '[Benchmark]\n[ArgumentsSource("Missing")]\nfunc Invalid(value: int) -> int => 1', 'Missing or ambiguous data source'),
        ('source-type', 'func Inputs() -> string[] => ["bad"]\n[Benchmark]\n[ArgumentsSource("Inputs")]\nfunc Invalid(value: int) -> int => 1', 'array of the input type'),
        ('duplicate-row', '[Benchmark]\n[Arguments(5)]\n[Arguments(5)]\nfunc Invalid(value: int) -> int => 1', 'duplicate data row'),
        ('parameters', '[Benchmark]\nfunc Invalid(value: int) -> int => 1', 'parameterized function needs a data attribute'),
        ('instance', 'class Fixture {\n[Benchmark]\nfunc Invalid() -> int => 1\n}', 'nongeneric module functions'),
        ('generic', '[Benchmark]\nfunc Invalid<T>() -> int => 1', 'nongeneric module functions'),
        ('result', '[Benchmark]\nfunc Invalid() -> bool => true', 'benchmarks require int'),
        ('empty', 'func NotABenchmark() -> int => 1', 'No benchmarks discovered'),
        ('named', '[Benchmark(Description: "Named description")]\nfunc Valid() -> int { System.Panic("Must not execute during discovery"); return 1 }', None),
    ]
    try:
        for name, body, expected in cases:
            folder = out/name
            folder.mkdir()
            source = folder/'Input.rvn'
            source.write_text('module DiscoveryValidation\nimport System.*\nimport NeoClr.Benchmarking.*\n'+body+'\nfunc Main() { }\n')
            registry = folder/'Registry.rvn'
            manifest = folder/'benchmarks.json'
            artifact = folder/'app.dll'
            command = [str(source) if part.endswith('/Discovery.rvn') else str(artifact)
                       if part == compile_template[compile_template.index('-o')+1] else part
                       for part in compile_template if not part.endswith('/BenchmarkRegistry.rvn')]
            result = subprocess.run(command,capture_output=True,text=True,timeout=120)
            if result.returncode:
                raise RuntimeError('Fixture compilation failed: '+result.stdout+result.stderr)
            # A failure must remove stale output from a prior successful invocation.
            registry.write_text('stale registry')
            manifest.write_text('{}')
            tail = discover_template.index('--')
            command = [*discover_template[:tail+1],str(artifact),str(registry),str(manifest),*discover_template[tail+4:]]
            result = subprocess.run(command,capture_output=True,text=True,timeout=120)
            if expected:
                if result.returncode == 0 or expected not in result.stderr or registry.exists() or manifest.exists():
                    raise RuntimeError('Invalid discovery accepted or left stale output: '+name+result.stderr)
            else:
                if result.returncode or json.loads(manifest.read_text())['benchmarks'][0]['description'] != 'Named description':
                    raise RuntimeError('Named description discovery failed: '+result.stderr)
            evidence['cases'].append(dict(name=name,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
        evidence['passed'] = True
    finally:
        (out/'validation.json').write_text(json.dumps(evidence,indent=2)+'\n')


if __name__ == '__main__':
    main()
