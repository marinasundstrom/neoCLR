"""Run only the application-level text foundation contract checks."""
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
parser.add_argument('--evidence', type=Path)
parser.add_argument('--runner', type=Path, help='measure_async host for the larger reader fixture')
parser.add_argument('--baseline-system', type=Path, help='Optional prior library for the identical boundary fixture')
selection = parser.add_mutually_exclusive_group()
selection.add_argument('--consumer-only', action='store_true')
selection.add_argument('--reader-only', action='store_true')
args = parser.parse_args()
if args.reader_only and not args.runner:
    parser.error("--reader-only requires a matching --runner for the larger fixtures")
root = Path(__file__).resolve().parent
artifacts = {name: getattr(args, name).resolve() for name in
             ('runtime', 'bridge', 'system', 'reference')}
if args.runner:
    artifacts['runner'] = args.runner.resolve()
if args.baseline_system:
    artifacts['baseline_system'] = args.baseline_system.resolve()
for path in artifacts.values():
    if not path.is_file():
        parser.error(f'Missing artifact: {path}')


def run(command, allowed_fault=None):
    result = subprocess.run([str(part) for part in command], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout, end='', flush=True)
    if result.returncode != 0 and not (allowed_fault and allowed_fault in result.stdout):
        result.check_returncode()
    return result.stdout


with tempfile.TemporaryDirectory(prefix='neoclr-text-boundaries-') as directory:
    work = Path(directory)
    for name in ('Main.rvn', 'Boundaries.rvn', 'Decoder.rvn', 'Consumer.rvn', 'Reader.rvn', 'ReaderMain.rvn', 'Boundaries.rvnproj'):
        shutil.copyfile(root / name, work / name)
    if args.reader_only:
        project = work / 'Boundaries.rvnproj'
        text = project.read_text()
        for name in ('Main.rvn', 'Boundaries.rvn', 'Decoder.rvn', 'Consumer.rvn'):
            text = text.replace(f'    <Compile Include="{name}" />\n', '')
        text = text.replace('<ItemGroup>', '<ItemGroup>\n    <Compile Include="ReaderMain.rvn" />\n    <Compile Include="Reader.rvn" />', 1)
        project.write_text(text)
    shutil.copyfile(artifacts['reference'], work / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', work / 'Boundaries.rvnproj', work / 'out'])
    app = work / 'out' / 'App.neoil'
    common = [app, '--system', artifacts['system']]
    run([artifacts['runtime'], 'verify', *common])
    results = {}
    runs = [('consumer', ['consumer'])]
    if args.reader_only:
        runs = [('reader', ['reader']), ('reader-boundary', ['reader-boundary']), ('reader-limit', ['reader-limit'])]
    elif not args.consumer_only:
        runs += [('contracts', []), ('splits-0-6', ['first']), ('splits-7-13', ['last'])]
    for label, guest_args in runs:
        if label in ('reader-boundary', 'reader-limit') and args.runner:
            output = run([args.runner.resolve(), app, artifacts['system'], '100000', '500000000', '--', *guest_args])
            if args.baseline_system:
                baseline = run([args.runner.resolve(), app, args.baseline_system.resolve(), '100000', '500000000', '--', *guest_args],
                               allowed_fault='ArrayLimitExceeded' if label == 'reader-limit' else None)
                results[label + '-comparison'] = {'current': output.splitlines(), 'baseline': baseline.splitlines(),
                                                  'note': 'Single bounded diagnostic run; not a throughput benchmark'}
        else:
            output = run([artifacts['runtime'], 'run', *common, '--', *guest_args])
        expected = ('Scalar and source-range contracts: passed',
                    'Decoding ownership, progress and errors: passed') if not guest_args else (
                    'Bounded UTF-8 split checks: passed',)
        if label == 'consumer':
            expected = ('Text API consumer: construction, extraction and split decoding passed',)
        if label == 'reader':
            expected = ('Incremental StreamReader contracts passed',)
        if label == 'reader-boundary':
            expected = ('Incremental StreamReader carry/output boundary passed',)
        if label == 'reader-limit':
            expected = ('Incremental StreamReader maximum bound passed',)
        if any(marker not in output for marker in expected):
            raise SystemExit(f'Missing success marker in {label}')
        results[label] = list(expected)
    if args.evidence:
        evidence = {'scope': ('Production StreamReader behavior; unchanged public signatures'
                              if args.reader_only else 'Application prototype; no System API change'),
                    'runs': results,
                    'sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                               for name, path in artifacts.items()}}
        args.evidence.write_text(json.dumps(evidence, indent=2) + '\n')
