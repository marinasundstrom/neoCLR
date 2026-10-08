"""Run separately compiled Math constants in the VM and record current AOT admission."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'bundle', 'runtime', 'aot', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    compiler, bundle, runtime, aot, output = (getattr(args, n).resolve() for n in ('compiler', 'bundle', 'runtime', 'aot', 'output'))
    output.mkdir(parents=True, exist_ok=False)
    source = ROOT / 'docs/experiments/raven-target/samples/library-math-constants.rvn'
    libraries = [bundle / (n + '.dll') for n in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web')]
    core, seed, ownership = [bundle / n for n in ('Core.dll', 'System.runtime.neox', 'ownership.json')]
    inputs = [Path(__file__), compiler, runtime, aot, source, core, seed, ownership, *libraries, *compiler.parent.glob('*.dll')]
    report = dict(revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, commands=[])

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=180)
        report['commands'].append(dict(command=result.args, status=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
        assert result.returncode == 0, report['commands'][-1]
        return result

    app = output / 'MathConstants.dll'
    run(['dotnet', compiler, 'neoclr', '--core-reference', core, '--runtime-seed', seed,
         '--bootstrap-intrinsics', '--bootstrap-ownership', ownership, '--object-library', 'System.Runtime',
         *[x for lib in libraries for x in ('--reference', lib)], '-o', app, source])
    context = ['--system', seed, *[x for lib in libraries for x in ('--module', lib)], '--object-root', libraries[0]]
    interpreted = run([runtime, 'run', app, *context, '--instructions', '100000000'])
    assert interpreted.stdout == 'Math constants passed\n' and not interpreted.stderr
    inspection = json.loads(run([aot, '--inspect', app, '@entry', '--closed-world', *context,
                                 '--compile-system', '--reference-arena', '--native-gc', '--bind-user-fault',
                                 '--bind-utf8-text', '--bind-console-write-line', '--bind-int32-to-string']).stdout)
    report['interpreter'] = 'passed'
    report['nativeAdmission'] = inspection['admission']
    report['nativeExecution'] = 'not run; this script records admission only'
    report['inputs'][str(app)] = hashlib.sha256(app.read_bytes()).hexdigest()
    for path in inputs:
        assert hashlib.sha256(path.read_bytes()).hexdigest() == report['inputs'][str(path)], path
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ('interpreter', 'nativeAdmission', 'nativeExecution')}, indent=2))


if __name__ == '__main__':
    main()
