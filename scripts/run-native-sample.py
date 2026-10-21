#!/usr/bin/env python3
"""Build and run one editable installed sample, retaining its declared JSON types."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def reflection_roots(runtime, assembly, names, run, constructors=False):
    if not names:
        return []
    with tempfile.TemporaryDirectory(prefix='neoclr-sample-reflection-') as directory:
        listing = Path(directory) / 'module.neoil'
        run([runtime, 'disassemble', assembly, listing])
        text = listing.read_text()
    metadata, _ = json.JSONDecoder().raw_decode(text[text.index('{'):])
    selected = []
    for name in names:
        suffix = '.T__' + name.encode().hex().upper()
        matches = [i for i, item in enumerate(metadata['types']) if item.get('origin', {}).get('name') == name or item['name'].endswith(suffix)]
        if len(matches) != 1:
            raise ValueError('Expected one reflection root: ' + name)
        selected.append(dict(definition=dict(module=metadata['name'], revision=metadata['revision'],
                                             index=matches[0]), construct=True, properties=True,
                             getters=True, setters=True))
    policy = dict(schemaVersion=2, types=selected)
    if constructors:
        for root in selected:
            root.update(properties=False, getters=False, setters=False, customAttributes=False,
                        fields=False, fieldGetters=False, fieldSetters=False, constructors=True)
        policy.update(schemaVersion=7, moduleCatalogs=[], moduleFunctions=[], typeCatalogs=[])
    path = assembly.with_suffix('.reflection.json')
    path.write_text(json.dumps(policy, indent=2) + '\n')
    return ['--reflection-roots', path]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('sample')
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.epilog = 'Pass program arguments after --, for example: http-json-client -- http://127.0.0.1:8080/'
    command_line = sys.argv[1:]
    separator = command_line.index('--') if '--' in command_line else len(command_line)
    args = parser.parse_args(command_line[:separator])
    program_arguments = command_line[separator + 1:]
    root = args.bundle.resolve()
    manifest = json.loads((root / 'manifest.json').read_text())
    if args.sample not in manifest['samples']:
        parser.error('Unknown packaged sample')
    folder = root / 'samples' / args.sample
    runtime = root / manifest['runtime']
    def run(command):
        return subprocess.run(list(map(str, command)), cwd=folder, check=True)
    run(['dotnet', root / 'sdk/tools/rvnc/rvnc.dll', 'neoclr', '--project', folder / 'App.rvnproj'])
    assembly = folder / 'bin/neoclr/App.dll'
    if manifest.get('jit', {}).get('default'):
        context = reflection_roots(runtime, assembly, manifest['samples'][args.sample].get('reflectionTypes', []), run)
    else:
        context = ['--execution', 'interpreter', '--system', root / manifest['runtimeSeed'],
                   '--instructions', '100000000']
        for module in manifest['modules']:
            context += ['--module', root / module]
        if manifest.get('objectRoot'):
            context += ['--object-root', root / manifest['objectRoot']]
    if program_arguments:
        context += ['--', *program_arguments]
    raise SystemExit(subprocess.call(list(map(str, [runtime, 'run', assembly, *context])), cwd=folder))


if __name__ == '__main__':
    main()
