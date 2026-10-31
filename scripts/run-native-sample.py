#!/usr/bin/env python3
"""Build and run one editable installed sample, retaining its declared JSON types."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def reflection_roots(runtime, assembly, names, run, constructors=False, methods=False, modules=(), attributes=False, module_attributes=()):
    if sum((constructors, methods, attributes, bool(module_attributes))) > 1:
        raise ValueError('Choose one retention capability')
    if not names and not module_attributes:
        return []
    def read_module(path):
        with tempfile.TemporaryDirectory(prefix='neoclr-sample-reflection-') as directory:
            listing = Path(directory) / 'module.neoil'
            run([runtime, 'disassemble', path, listing])
            text = listing.read_text()
        return json.JSONDecoder().raw_decode(text[text.index('{'):])[0]

    inventories = [read_module(assembly)]
    selected = []
    for name in names:
        suffix = '.T__' + name.encode().hex().upper()
        def candidates():
            return [(metadata, i) for metadata in inventories for i, item in enumerate(metadata['types'])
                    if item.get('origin', {}).get('name') == name or item['name'].endswith(suffix)]
        matches = candidates()
        if not matches and len(inventories) == 1:
            inventories.extend(read_module(path) for path in modules)
            matches = candidates()
        if len(matches) != 1:
            raise ValueError('Expected one reflection root: ' + name)
        metadata, index = matches[0]
        selected.append(dict(definition=dict(module=metadata['name'], revision=metadata['revision'],
                                             index=index), construct=True, properties=True,
                             getters=True, setters=True))
    policy = dict(schemaVersion=2, types=selected)
    if constructors:
        for root in selected:
            root.update(properties=False, getters=False, setters=False, customAttributes=False,
                        fields=False, fieldGetters=False, fieldSetters=False, constructors=True)
        policy.update(schemaVersion=7, moduleCatalogs=[], moduleFunctions=[], typeCatalogs=[])
    if methods:
        policy = dict(schemaVersion=8, types=[], moduleCatalogs=[], moduleFunctions=[],
                      typeCatalogs=[], methodTypes=[root['definition'] for root in selected])
    if attributes or module_attributes:
        for root in selected:
            root.update(construct=False, properties=False, getters=False, setters=False,
                        customAttributes=True, fields=False, fieldGetters=False,
                        fieldSetters=False, constructors=False)
        policy = dict(schemaVersion=8, types=selected, moduleCatalogs=[], moduleFunctions=[],
                      typeCatalogs=[], methodTypes=[])
        if module_attributes:
            inventories.extend(read_module(path) for path in modules)
            for entry in module_attributes:
                matches = [owner for metadata in inventories for owner in metadata.get('assemblies', [])
                           if entry['assembly'] in (owner['name'], owner['full_name'])
                           and entry['name'] in owner.get('declaration_modules', {}).get('names', [])]
                if len(matches) != 1:
                    raise ValueError('Expected one declared module: ' + entry['assembly'] + ':' + entry['name'])
                identity = matches[0]['full_name']
                if identity not in policy['moduleCatalogs']:
                    policy['moduleCatalogs'].append(identity)
                policy['moduleFunctions'].append(dict(assembly=identity, name=entry['name'], customAttributes=True))
    path = assembly.with_suffix('.reflection.json')
    path.write_text(json.dumps(policy, indent=2) + '\n')
    return ['--reflection-roots', path]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('sample')
    parser.add_argument('--execution', choices=('jit', 'interpreter'),
                        help='Explicit execution mode; otherwise use the bundle default')
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
    if args.execution == 'jit' or (args.execution is None and manifest.get('jit', {}).get('default')):
        context = ['--execution', 'jit', *reflection_roots(
            runtime, assembly, manifest['samples'][args.sample].get('reflectionTypes', []), run)]
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
