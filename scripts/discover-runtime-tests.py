#!/usr/bin/env python3
"""Inspect native test metadata and generate typed Raven registration (no guest execution)."""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def discover(project, bundle, output):
    project, bundle, output = project.resolve(), bundle.resolve(), output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    registry = output / 'TestRegistry.rvn'
    registry.write_text('module NeoClr.Testing\nfunc RegisterDiscoveredTests(suite: TestSuite) { }\n')
    env = dict(os.environ, NeoClrBundleRoot=str(bundle), NeoClrTestRegistry=str(registry))
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    command = ['dotnet', str(compiler), 'neoclr', '--project', str(project)]
    compiled = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding="utf-8", timeout=300)
    (output / 'compile.stdout.log').write_text(compiled.stdout)
    (output / 'compile.stderr.log').write_text(compiled.stderr)
    if compiled.returncode:
        registry.unlink(missing_ok=True)
        raise RuntimeError('Test discovery compilation failed; see ' + str(output))
    paths = [line.removeprefix('Native build output: ') for line in compiled.stdout.splitlines()
             if line.startswith('Native build output: ')]
    if not paths or not Path(paths[-1]).is_absolute() or not Path(paths[-1]).is_file():
        registry.unlink(missing_ok=True)
        raise RuntimeError('Compiler did not report a native test assembly')
    artifact = Path(paths[-1])
    catalog = json.loads((bundle / 'lib/bundle.json').read_text())
    command = ['dotnet', 'run', '--project', str(ROOT / 'tools/testing/NeoCLR.TestDiscovery'),
               '-p:WarningLevel=0', '--', str(artifact), str(registry), str(output / 'tests.json'),
               *[str(bundle / 'lib' / (name + '.dll')) for name in catalog['assemblyNames']]]
    discovered = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=300)
    (output / 'discovery.stdout.log').write_text(discovered.stdout)
    (output / 'discovery.stderr.log').write_text(discovered.stderr)
    if discovered.returncode:
        registry.unlink(missing_ok=True)  # A failed discovery must never leave a runnable empty registry.
        raise RuntimeError(discovered.stderr.strip() or 'Test discovery failed')
    return registry


def verify_registration(artifact, bundle, discovery_output):
    """The final compiled artifact must rediscover the same IDs/descriptions/adapters."""
    registry = discovery_output / 'VerifiedRegistry.rvn'
    manifest = discovery_output / 'verified-tests.json'
    catalog = json.loads((bundle / 'lib/bundle.json').read_text())
    command = ['dotnet', 'run', '--no-build', '--project', str(ROOT / 'tools/testing/NeoCLR.TestDiscovery'),
               '--', str(artifact), str(registry), str(manifest),
               *[str(bundle / 'lib' / (name + '.dll')) for name in catalog['assemblyNames']]]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', timeout=60)
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or 'Final artifact discovery failed')
    if (manifest.read_bytes() != (discovery_output / 'tests.json').read_bytes() or
            registry.read_bytes() != (discovery_output / 'TestRegistry.rvn').read_bytes()):
        raise RuntimeError('Final compilation changed the discovered test contract')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(discover(args.project, args.bundle, args.output))
