"""Verify full-library admission with an explicitly finalized retained seed.

The application is a small API-authored control returning 42, not proof of full
System API execution. The aggregate library is nevertheless loaded and verified.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('audit', 'runtime', 'app', 'module', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    audit = args.audit.resolve()
    report = json.loads((audit / 'audit.json').read_text())
    case = next(c for c in report['cases'] if c['name'] == 'full-owned-handle')
    if case['outcome'] != 'emitted' or case['runtimeSeedFinalization']['exitCode'] != 0:
        raise ValueError('Successful full-source emission and runtime seed finalization required')
    seed = Path(case['runtimeSeed'])
    owner = seed.parent / 'Numbers.dll'
    retained = seed.parent / 'System.retained.json'
    runtime, app, supporting = (getattr(args, n).resolve() for n in ('runtime', 'app', 'module'))
    commands = []

    def run(command, expected=0, contains=None):
        result = subprocess.run([str(p) for p in command], cwd=ROOT, capture_output=True, text=True, timeout=180)
        commands.append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        if result.returncode != expected or contains and contains not in result.stdout + result.stderr:
            raise RuntimeError(json.dumps(commands[-1], indent=2))
        return result

    dependencies = ['--module', supporting, '--module', owner, '--object-root', owner]
    run([runtime, 'verify', app, '--system', seed] + dependencies)
    executed = run([runtime, 'run', app, '--system', seed] + dependencies, 42)
    if executed.stdout or executed.stderr:
        raise RuntimeError('Unexpected control execution output')
    # The disassembler emits the serialized module header as JSON. This is only
    # negative-test preparation; production finalization uses metadata reader APIs.
    listing = output / 'seed.dis'
    run([runtime, 'disassemble', seed, listing])
    model, _ = json.JSONDecoder().raw_decode(listing.read_text().split('.module ', 1)[1])
    wrong = json.loads(retained.read_text())
    wrong['references'] = model['references']
    if len(wrong['references']) != 1 or wrong['references'][0]['revision'] == '0.0.0.0':
        raise ValueError('Expected one revisioned source-owner dependency')
    wrong['references'][0]['revision'] = '0.0.0.0'
    wrong_json, wrong_seed = output / 'wrong-revision.json', output / 'wrong-revision.neox'
    wrong_json.write_text(json.dumps(wrong))
    translate = ['dotnet', 'run', '--project', ROOT / 'tools/metadata/NeoCLR.Metadata.Translate', '--']
    run(translate + [wrong_json, wrong_seed])
    run([runtime, 'verify', app, '--system', wrong_seed] + dependencies, 1, 'module revision mismatch')
    bad = output / 'bad.dll'
    bad.write_bytes(b'invalid artifact')
    for name, references in [('duplicate', [owner, owner]), ('missing', [output / 'absent.dll']), ('malformed', [bad])]:
        artifact = output / (name + '.neox')
        run(translate + [retained, artifact] + [part for p in references for part in ('--reference', p)], 1)
        if artifact.exists():
            raise RuntimeError('Failed reference validation published output')
    inputs = [Path(__file__), audit / 'audit.json', owner, seed, retained, runtime, app, supporting,
              ROOT / 'tools/metadata/NeoCLR.Metadata.Translate/Program.cs']
    evidence = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                    commands=commands, hashes={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs})
    (output / 'validation.json').write_text(json.dumps(evidence, indent=2) + '\n')
    print(output / 'validation.json')


if __name__ == '__main__':
    main()
