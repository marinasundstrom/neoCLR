"""Verify the bounded Raven match matrix and execute every admitted fixture."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('probe', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--system', type=Path, help='Matching preview runtime library')
args = parser.parse_args()
results = json.loads((args.probe / 'match-results.json').read_text())
accepted = {
    'Positional': '42\n-1\n',
    'NominalDeconstruction': '42\n',
    'CaseImports': '42\n-1\n',
    'OptionPositional': '42\n-1\n',
    'PositionalSingleEvaluation': '7\n42\n',
    'Forms': '42\n-1\nPresent\nAbsent\n42\nOverflow\n',
    'Guard': '1\n2\n3\n',
    'StatementTail': '42\n-1\n',
    'SingleEvaluation': '7\n42\n',
    'VoidOutput': '42\nSaved\nCompleted\nOverflow\n',
    'ExpressionBlockReturn': '42\n',
}
rejected = {'PositionalWrongArity': 'RAV1610',
            'StatementReturn': 'RAV1503', 'MissingExpression': 'RAV2100',
            'MissingStatement': 'RAV2100', 'UnreachableArm': 'RAV2101', 'WrongCase': 'RAV2102'}
if set(results) != set(accepted) | set(rejected):
    raise AssertionError('Review the match matrix when adding or removing cases')
for name, expected in accepted.items():
    if results[name]['Stage'] != 'imported':
        raise AssertionError((name, results[name]))
    artifact = args.probe / (name + '.neoil')
    for command in ('verify', 'run'):
        options = ['--system', str(args.system.resolve())] if args.system else []
        run = subprocess.run([str(args.runtime.resolve()), command, str(artifact.resolve()), *options],
                             capture_output=True, text=True, timeout=45)
        if run.returncode or (command == 'run' and run.stdout != expected):
            raise AssertionError((name, command, run.stdout, run.stderr))
    results[name]['Output'] = expected.splitlines()
for name, diagnostic in rejected.items():
    result = results[name]
    if result['Stage'] != 'compile-rejected' or diagnostic not in [d['Id'] for d in result['Diagnostics']]:
        raise AssertionError((name, result))
    if (args.probe / (name + '.neoil')).exists():
        raise AssertionError('Rejected source produced an executable: ' + name)
if not (args.probe / 'UninitializedString.dll').exists() or (args.probe / 'UninitializedString.neoil').exists():
    raise AssertionError('Missing uninitialized-string rejection evidence')
print(json.dumps(results, indent=2))
