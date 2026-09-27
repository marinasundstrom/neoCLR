"""Compile and run only the bounded builder evaluation and identical-output construction diagnostics."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument('--evidence', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parent
artifacts = {name: getattr(args, name).resolve() for name in
             ('runtime', 'bridge', 'system', 'reference', 'runner')}
for path in artifacts.values():
    if not path.is_file():
        parser.error(f'Missing artifact: {path}')


def run(command):
    result = subprocess.run([str(part) for part in command], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end='', flush=True)
    result.check_returncode()
    return result.stdout


with tempfile.TemporaryDirectory(prefix='neoclr-encoding-selection-') as directory:
    work = Path(directory)
    for name in ('Builder.rvn', 'BuilderMain.rvn'):
        shutil.copyfile(root / name, work / name)
    project = (root / 'Boundaries.rvnproj').read_text()
    for name in ('Main.rvn', 'Boundaries.rvn', 'Decoder.rvn', 'Consumer.rvn'):
        project = project.replace(f'    <Compile Include="{name}" />\n', '')
    project = project.replace('<ItemGroup>', '<ItemGroup>\n    <Compile Include="Builder.rvn" />\n    <Compile Include="BuilderMain.rvn" />', 1)
    (work / 'Builder.rvnproj').write_text(project)
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Builder.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']])
    results = {}
    output = run([artifacts['runtime'], 'run', app, '--system', artifacts['system'], '--', 'contracts'])
    assert 'Bounded report construction contracts passed' in output
    results['contracts'] = 'passed'
    invalid = subprocess.run([str(artifacts['runtime']), 'run', str(app), '--system', str(artifacts['system']), '--', 'invalid-limit'], text=True, capture_output=True)
    assert invalid.returncode != 0 and 'Builder limit must be between' in invalid.stderr, invalid.stdout + invalid.stderr
    results['invalid-limit'] = 'expected fault before use'
    # No timing assertions: measure only this workload, alternating order per repeat.
    import re
    diagnostics = []
    for size, count in [('small', 8), ('large', 1024)]:
        for repeat in range(3):
            for mode in (['concat', 'builder'] if repeat % 2 == 0 else ['builder', 'concat']):
                command = [artifacts['runner'], app, artifacts['system'], '100000', '500000000', '--', mode, size]
                measured = subprocess.run([str(p) for p in command], text=True, capture_output=True)
                measured.check_returncode()
                assert measured.stdout.strip() == '0123456789abcdef' * count, measured.stdout[:200]
                metrics = dict((key, int(value)) for key, value in re.findall(r'(\w+)=(\d+)', measured.stderr))
                assert 'run_ms' in metrics, measured.stderr
                diagnostics.append({'size': size, 'parts': count, 'mode': mode, 'repeat': repeat, **metrics})
                print(size, mode, metrics, flush=True)
    evidence = {'scope': 'Application-only builder evaluation; no public API addition',
                'contracts': results, 'diagnostics': diagnostics,
                'notes': ['Identical final bytes verified in every diagnostic invocation.',
                          'run_ms includes guest append, materialization, checks and output; excludes load/verify.',
                          'Three fresh invocations per case, alternating order; no warm throughput claim.',
                          'GC object counts do not measure native String payload allocations or bytes.'],
                'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in artifacts.items()},
                'sourceSha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in ['Builder.rvn', 'BuilderMain.rvn']}}
    args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
