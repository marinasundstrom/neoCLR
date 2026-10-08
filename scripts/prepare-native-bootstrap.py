#!/usr/bin/env python3
"""Generate explicit primitive-core and retained-seed inputs without legacy library artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def retained_model(model):
    """Remove the temporary assembler Object root and stale dense definition IDs."""
    if sum(t['name'] == 'System.Object' for t in model['types']) != 1:
        raise ValueError('Expected exactly one bootstrap Object declaration')
    model['types'] = [t for t in model['types'] if t['name'] != 'System.Object']
    model['functions'] = [f for f in model['functions'] if f.get('owner') != {'Named': 'System.Object'}]

    def clear_ids(value):
        if isinstance(value, dict):
            definition = value.get('definition')
            if isinstance(definition, dict) and {'module', 'index'} <= set(definition) <= {'module', 'revision', 'index'}:
                del value['definition']
            for child in value.values():
                clear_ids(child)
        elif isinstance(value, list):
            for child in value:
                clear_ids(child)
    clear_ids(model)
    return model


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('probe', 'runtime', 'translator', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    ownership_path = ROOT / 'runtime/raven/projects/System.Runtime/ownership.json'
    ownership = json.loads(ownership_path.read_text())
    if ownership['unit'] != {'assemblyName': 'System.Runtime', 'typeName': 'System.Void'}:
        raise ValueError('Unsupported unit ownership')
    for name in ('System.Value', 'System.RuntimeTypeHandle'):
        if ownership['nativePrimitives'].get(name) != 'System.Runtime':
            raise ValueError('Unsupported source primitive ownership: ' + name)
    source = ROOT / 'runtime/raven/native/poc-seed.neoil'
    seed_text = source.read_text()
    for name in ('System.RuntimeTypeHandle', 'System.Void', 'System.Value'):
        declaration = '.type ' + name + '\n.sealed\n.end\n'
        if seed_text.count(declaration) != 1:
            raise ValueError('Expected exactly one retained declaration: ' + name)
        seed_text = seed_text.replace(declaration, '')
    for path in (args.probe, args.runtime, args.translator):
        if not path.is_file():
            raise FileNotFoundError(path)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = dict(kind='explicit-source-runtime-bootstrap', sourceRevision=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), commands=[])

    def run(command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=120)
        report['commands'].append(dict(command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        (output / 'preparation-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)

    core = output / 'Core.dll'
    # This narrow CLI bootstrap excludes Fail and Math: the native source library owns them.
    run(['dotnet', args.probe.resolve(), '--reference-source-runtime-core', core])
    seed_source = output / 'System.neoil'
    seed_source.write_text(seed_text)
    bootstrap_json = output / 'System.bootstrap.json'
    run([args.runtime.resolve(), 'assemble', seed_source, bootstrap_json, '--format', 'json'])
    retained = output / 'System.retained.json'
    retained.write_text(json.dumps(retained_model(json.loads(bootstrap_json.read_text())), separators=(',', ':')))
    seed = output / 'System.neox'
    run(['dotnet', args.translator.resolve(), retained, seed])
    inputs = [Path(__file__), source, ownership_path, args.probe, args.runtime, args.translator]
    inputs += list((ROOT / 'docs/experiments/raven-target').glob('*.cs'))
    inputs += list(args.probe.parent.glob('*.dll')) + list(args.translator.parent.glob('*.dll'))
    outputs = [core, seed_source, bootstrap_json, retained, seed]
    report['hashes'] = {str(path.resolve()): sha(path) for path in inputs + outputs}
    (output / 'preparation-evidence.json').write_text(json.dumps(report, indent=2) + '\n')
    # Publish only after every generator succeeds. The native seed is finalized
    # against the emitted Runtime identity later by build-native-class-library.py.
    manifest = dict(version=1, kind=report['kind'], core=core.name, compileTimeSeed=seed.name,
                    retainedModel=retained.name, objectAssembly='System.Runtime',
                    files={path.name: sha(path) for path in outputs})
    (output / 'bootstrap.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print('Prepared source-owned bootstrap inputs: ' + str(output))


if __name__ == '__main__':
    main()
