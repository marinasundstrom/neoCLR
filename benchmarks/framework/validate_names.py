#!/usr/bin/env python3
"""Qualify escaped benchmark discovery, exact selectors and execution in three modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bundle, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    inputs = [Path(__file__), ROOT / 'tools/discovery/RavenNames.cs',
              *sorted((ROOT / 'tools/benchmarking/NeoCLR.BenchmarkDiscovery').glob('*.cs*')),
              *sorted((ROOT / 'benchmarks/framework').glob('*.rvn'))]
    report = dict(passed=False, commands=[], inputs={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},
                  bundleManifestSha256=hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest())
    env = dict(os.environ, NeoClrBundleRoot=str(bundle))

    def run(command):
        result = subprocess.run(list(map(str, command)), env=env, cwd=ROOT,
                                capture_output=True, text=True, timeout=300)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode,
                                       stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    try:
        registry = out / 'BenchmarkRegistry.rvn'
        registry.write_text('module NeoClr.Benchmarking\nfunc RegisterDiscoveredBenchmarks(suite: BenchmarkSuite) { }\n')
        project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        ET.SubElement(project, 'Import', Project='$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props')
        properties = ET.SubElement(project, 'PropertyGroup')
        for key, value in dict(TargetFramework='net10.0', OutputType='Exe', AssemblyName='BenchmarkNames', EnableDefaultCompileItems='false').items():
            ET.SubElement(properties, key).text = value
        items = ET.SubElement(project, 'ItemGroup')
        for path in [ROOT / 'benchmarks/framework' / name for name in ('Discovery.rvn', 'Worker.rvn', 'BenchmarkAttribute.rvn')] + [registry]:
            ET.SubElement(items, 'Compile', Include=str(path))
        project_path = out / 'Benchmarks.rvnproj'
        ET.ElementTree(project).write(project_path, encoding='unicode')
        compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
        stdout = run(['dotnet', compiler, 'neoclr', '--project', project_path])
        artifact = Path(next(line.removeprefix('Native build output: ') for line in stdout.splitlines() if line.startswith('Native build output: ')))
        catalog = json.loads((bundle / 'lib/bundle.json').read_text())
        dependencies = [bundle / 'lib' / (name + '.dll') for name in catalog['assemblyNames']]
        discovery = ROOT / 'tools/benchmarking/NeoCLR.BenchmarkDiscovery'
        run(['dotnet', 'run', '--project', discovery, '-p:WarningLevel=0', '--', artifact, registry, out / 'benchmarks.json', *dependencies])
        native = out / 'native'
        suffix = '.exe' if os.name == 'nt' else ''
        codegen = bundle / ('bin/neoclr-codegen' + suffix)
        runtime = bundle / ('bin/neoclr' + suffix)
        host_suffix = '.dll' if os.name == 'nt' else '.dylib' if sys.platform == 'darwin' else '.so'
        run([sys.executable, ROOT / 'scripts/build-native-project.py', '--project', project_path,
             '--bundle', bundle, '--aot', codegen, '--output', native,
             '--profile', 'windows-console' if os.name == 'nt' else 'console'])
        run(['dotnet', 'run', '--no-build', '--project', discovery, '--', native / 'app.dll',
             out / 'VerifiedRegistry.rvn', out / 'verified-benchmarks.json', *dependencies])
        for first, second in [('BenchmarkRegistry.rvn', 'VerifiedRegistry.rvn'), ('benchmarks.json', 'verified-benchmarks.json')]:
            if (out / first).read_bytes() != (out / second).read_bytes():
                raise RuntimeError('Final artifact changed discovery: ' + first)
        base = [runtime, 'run', native / 'app.dll', '--system', bundle / 'lib' / catalog['runtimeSeed'],
                *[arg for path in dependencies for arg in ('--module', path)], '--object-root', bundle / 'lib/System.Runtime.dll']
        commands = dict(interpreter=base + ['--execution', 'interpreter', '--'],
                        jit=base + ['--jit-host', bundle / ('bin/neoclr-jit-host' + host_suffix), '--jit-compiler', codegen, '--'],
                        native=[native / ('app' + suffix)])
        names = ['manual', 'Benchmarks.Discovery.Accept source rows/Benchmarks.Discovery.Inputs with spaces. λ[0]',
                 'Benchmarks.Discovery.Accept source rows/Benchmarks.Discovery.Inputs with spaces. λ[1]',
                 'Benchmarks.Discovery.Add café. 世界 😀(5)', 'Benchmarks.Discovery.Add café. 世界 😀(9)',
                 'Benchmarks.Discovery.Multiply', 'Benchmarks.Discovery.Sum a sequence of integers']
        report['cases'] = names
        for mode, command in commands.items():
            if run(command + ['--list']) != 'NEOBENCH/1 CASES\n' + '\n'.join(names) + '\nNEOBENCH/1 END\n':
                raise RuntimeError(mode + ': case inventory differs')
            for name in names:
                if run(command + [name, '1', '2']) != 'NEOBENCH/1 READY\nNEOBENCH/1 BEGIN\nNEOBENCH/1 END\n2\n':
                    raise RuntimeError(mode + ': incorrect result for ' + name)
        for path in inputs:
            if hashlib.sha256(path.read_bytes()).hexdigest() != report['inputs'][str(path.relative_to(ROOT))]:
                raise RuntimeError('Input changed during validation: ' + str(path))
        report['passed'] = True
    finally:
        (out / 'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print('Benchmark escaped names: PASS (interpreter, JIT, native)')


if __name__ == '__main__':
    main()
