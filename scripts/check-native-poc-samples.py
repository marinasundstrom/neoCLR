#!/usr/bin/env python3
"""Inventory selected unchanged POC samples through the native compiler driver with optional bounded execution controls."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SAMPLES = {
    'native-async-propagation': ['extended-cli-metadata/bootstrap/native-async-propagation.rvn'],
    'native-async-state': ['extended-cli-metadata/bootstrap/native-async-state.rvn'],
    **{name: [f'raven-target/samples/{name}.rvn'] for name in (
        'application-order-collections', 'application-types', 'application-interfaces',
        'application-inheritance', 'library-async', 'library-async-default-queue',
        'library-async-cancellation')},
    'json-object-mapping': ['json-object-mapping/Mapping.rvn', 'json-object-mapping/Main.rvn'],
    'http-json-server': ['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpServer.rvn'],
    'http-json-client': ['json-object-mapping/HttpApplication.rvn', 'json-object-mapping/HttpClient.rvn'],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'ownership', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--reference', type=Path, action='append', required=True)
    parser.add_argument('--runtime', type=Path, help='Execute the established non-network controls after successful compilation')
    parser.add_argument('--async-library', help='Explicit native Task/builder assembly identity')
    parser.add_argument('--case', choices=SAMPLES, action='append', help='Limit the inventory to selected cases')
    args = parser.parse_args()
    paths = {name: getattr(args, name).resolve() for name in ('compiler', 'core', 'seed', 'ownership', 'output')}
    refs = [p.resolve() for p in args.reference]
    paths['output'].mkdir(parents=True, exist_ok=False)
    sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    inputs = [paths[k] for k in ('compiler', 'core', 'seed', 'ownership')] + refs
    inputs += [paths['compiler'].parent / n for n in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    runtime = args.runtime.resolve() if args.runtime else None
    if runtime:
        inputs.append(runtime)
    expected = {
        'native-async-propagation': 'Native async propagation checks passed\n',
        'native-async-state': 'Native async state checks passed\n',
        'library-async-cancellation': 'Cancelled\n',
        'application-order-collections': (ROOT / 'docs/experiments/raven-target/samples/application-order-collections.expected.txt').read_text(),
        'application-interfaces': '42\n99\n',
        'json-object-mapping': 'JSON object mapping checks passed\n',
    }
    report = {'scope': 'selected unchanged POC samples; only cases with an execution record claim runtime validation',
              'revisions': {str(p): subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=p, text=True).strip()
                            for p in (ROOT, paths['compiler'].parent)},
              'inputs': {str(p): sha(p) for p in inputs}, 'cases': []}
    common = ['dotnet', str(paths['compiler']), 'neoclr', '--core-reference', str(paths['core']),
              '--runtime-seed', str(paths['seed']), '--bootstrap-intrinsics', '--bootstrap-ownership', str(paths['ownership'])]
    if args.async_library:
        common += ['--async-library', args.async_library]
    for reference in refs:
        common += ['--reference', str(reference)]
    selected_samples = {name: SAMPLES[name] for name in args.case} if args.case else SAMPLES
    for name, sources in selected_samples.items():
        source_paths = [ROOT / 'docs/experiments' / p for p in sources]
        artifact = paths['output'] / (name + '.dll')
        command = common + ['-o', str(artifact)] + [str(p) for p in source_paths]
        try:
            result = subprocess.run(command, capture_output=True, text=True, timeout=120)
            case = dict(name=name, command=command, exitCode=result.returncode,
                        stdout=result.stdout, stderr=result.stderr, compiled=result.returncode == 0 and artifact.exists())
        except subprocess.TimeoutExpired:
            case = dict(name=name, command=command, compiled=False, timeoutSeconds=120)
        case['sources'] = {str(p.relative_to(ROOT)): sha(p) for p in source_paths}
        case['published'] = artifact.exists()
        if artifact.exists():
            case['artifactSha256'] = sha(artifact)
        if runtime and case['compiled'] and name in expected:
            command = [str(runtime), 'run', str(artifact), '--system', str(paths['seed']), '--instructions', '100000000']
            for reference in refs:
                command += ['--module', str(reference)]
            try:
                execution = subprocess.run(command, capture_output=True, text=True, timeout=120)
                case['execution'] = dict(command=command, exitCode=execution.returncode,
                                         stdout=execution.stdout, stderr=execution.stderr, expectedStdout=expected[name],
                                         passed=execution.returncode == 0 and execution.stdout == expected[name] and not execution.stderr)
            except subprocess.TimeoutExpired:
                case['execution'] = dict(command=command, passed=False, timeoutSeconds=120)
        report['cases'].append(case)
        (paths['output'] / 'inventory.json').write_text(json.dumps(report, indent=2) + '\n')
        print(name + ': ' + ('compiled' if case['compiled'] else 'blocked'), flush=True)
    print(f"Compiled {sum(c['compiled'] for c in report['cases'])}/{len(selected_samples)}; inventory: {paths['output'] / 'inventory.json'}")
    # This is an inventory command, not a green acceptance gate. Failures are in the report.


if __name__ == '__main__':
    main()
