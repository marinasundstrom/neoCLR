#!/usr/bin/env python3
"""Generate explicit primitive-core and retained-seed inputs without legacy library artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import shutil
import re

ROOT = Path(__file__).resolve().parent.parent

SOURCE_TEXT_SERVICES = (
    'Int32ToString', 'Int64ToString', 'StringFromChars',
    'StringGraphemeAt', 'StringIntern', 'StringConcat',
    'StringCompareOrdinal', 'StringCompareOrdinalIgnoreCase', 'StringReplaceOrdinal',
    'StringContainsOrdinal', 'StringStartsWithOrdinal', 'StringEndsWithOrdinal',
    'StringGraphemeCount', 'CharFromString', 'CharText',
    'CharCategory', 'StringByteCount', 'StringSliceUtf8',
    'StringToUpperInvariant', 'StringToLowerInvariant', 'StringHashOrdinalIgnoreCase',
    'ParseBoolean', 'ParseSByte', 'ParseByte',
    'ParseInt16', 'ParseUInt16', 'ParseInt32',
    'ParseUInt32', 'ParseInt64', 'ParseUInt64',
    'ParseSingle', 'ParseDouble', 'Utf8Decode',
)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def retained_model(model):
    """Remove the temporary assembler Object root and stale dense definition IDs."""
    if sum(t['name'] == 'System.Object' for t in model['types']) != 1:
        raise ValueError('Expected exactly one bootstrap Object declaration')
    model['types'] = [t for t in model['types'] if t['name'] != 'System.Object']
    model['functions'] = [f for f in model['functions'] if f.get('owner') != {'Named': 'System.Object'}]

    # Production Console is source-owned; its obsolete seed facade would retain
    # a cross-assembly call to the now-internal native formatting service.
    if sum(t['name'] == 'System.Console' for t in model['types']) != 1:
        raise ValueError('Expected exactly one bootstrap Console declaration')
    model['types'] = [t for t in model['types'] if t['name'] != 'System.Console']
    model['functions'] = [f for f in model['functions'] if f.get('owner') != {'Named': 'System.Console'}]

    source_services = {'neoCLR.Runtime.' + name for name in
                       ('TypeMetadataToken', 'TypeDisplayName', 'TypeShape', 'TypeArgument',
                        'TypeArgumentCount', 'TypeEquals', 'TypeName') + SOURCE_TEXT_SERVICES}
    for name in source_services:
        if sum(f['name'] == name for f in model['functions']) != 1:
            raise ValueError('Expected exactly one retained native service: ' + name)
    model['functions'] = [f for f in model['functions'] if f['name'] not in source_services]

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
    source_core = parser.add_mutually_exclusive_group(required=True)
    source_core.add_argument('--probe', type=Path)
    source_core.add_argument('--core-reference', type=Path, help='Reuse an explicitly selected primitive-only Core.dll from a qualified native bundle')
    for name in ('runtime', 'translator', 'output'):
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
    marker = '.type class System.ParamArrayAttribute\n.method instance .ctor() -> noresult\nret\n.end\n.end\n'
    if 'System.ParamArrayAttribute' in ownership['libraries'][0]['types']:
        if seed_text.count(marker) != 1:
            raise ValueError('Expected exactly one retained parameter-array marker')
        seed_text = seed_text.replace(marker, '')
    # Source runtime declarations now own all three generic queue services. Remove
    # both legacy facade methods and seed implementations, with exact shape checks.
    for name in ('GetDefaultTaskQueue', 'GetCurrentTaskQueue', 'RegisterTaskQueue'):
        register = name == 'RegisterTaskQueue'
        parameters = 'T queue' if register else ''
        result = 'noresult' if register else 'T'
        body = ('ldarg queue\n' if register else '') + f'call neoCLR.Runtime.{name}<T>({"T" if register else ""})\n'
        body += ('pop\n' if register else '') + 'ret\n'
        facade = f'.method static {name}<T>({parameters}) -> {result}\n{body}.end\n'
        service = f'.function neoCLR.Runtime.{name}<T>({parameters}) -> {"Void" if register else "T"}\n.methodimpl InternalCall\n.end\n'
        for declaration in (facade, service):
            if seed_text.count(declaration) != 1:
                raise ValueError('Expected exactly one retained queue declaration: ' + name)
            seed_text = seed_text.replace(declaration, '')
    # Native Value ownership supplies these managed bodies, not InternalCalls.
    for name, result, opcode in (('IsValue', 'Boolean', 'value.is'), ('UnpackValue', 'T', 'value.unpack')):
        declaration = f'.method static {name}<T>(Value value) -> {result}\nldarg value\n{opcode} T\nret\n.end\n'
        if seed_text.count(declaration) != 1:
            raise ValueError('Expected exactly one retained payload helper: ' + name)
        seed_text = seed_text.replace(declaration, '')
    # Native System.Runtime owns these handle/text primitives. Remove the old facade;
    # seed service declarations remain available to assemble the temporary Object
    # body and are removed from the retained model after that body is discarded.
    for name in ('TypeMetadataToken', 'TypeDisplayName', 'TypeShape', 'TypeArgument',
                 'TypeArgumentCount', 'TypeEquals', 'TypeName') + SOURCE_TEXT_SERVICES:
        pattern = r'(?m)^\.method static ' + name + r'\([^\n]*\n.*?^\.end\n'
        seed_text, count = re.subn(pattern, '', seed_text, flags=re.DOTALL)
        if count != 1:
            raise ValueError('Expected exactly one retained native facade: ' + name)
    for path in (args.probe or args.core_reference, args.runtime, args.translator):
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
    original_core = output / 'Core.bootstrap.dll'
    # This narrow CLI bootstrap excludes Fail and Math: the native source library owns them.
    if args.probe:
        run(['dotnet', args.probe.resolve(), '--reference-source-runtime-core', original_core])
    else:
        shutil.copyfile(args.core_reference.resolve(), original_core)
        report['reusedPrimitiveCore'] = str(args.core_reference.resolve())
    run(['dotnet', args.translator.resolve(), '--prepare-source-attribute-core', original_core, core])
    seed_source = output / 'System.neoil'
    seed_source.write_text(seed_text)
    bootstrap_json = output / 'System.bootstrap.json'
    run([args.runtime.resolve(), 'assemble', seed_source, bootstrap_json, '--format', 'json'])
    retained = output / 'System.retained.json'
    retained.write_text(json.dumps(retained_model(json.loads(bootstrap_json.read_text())), separators=(',', ':')))
    seed = output / 'System.neox'
    run(['dotnet', args.translator.resolve(), retained, seed])
    inputs = [Path(__file__), source, ownership_path, args.probe or args.core_reference, args.runtime, args.translator]
    if args.probe:
        inputs += list((ROOT / 'docs/experiments/raven-target').glob('*.cs'))
        inputs += list(args.probe.parent.glob('*.dll'))
    inputs += list(args.translator.parent.glob('*.dll'))
    outputs = [core, original_core, seed_source, bootstrap_json, retained, seed]
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
