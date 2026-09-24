#!/usr/bin/env python3
"""Inspect generic union companion metadata and compile a separate Raven consumer."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--bridge', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
bridge = args.bridge.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], env=env, capture_output=True,
                            text=True, timeout=180)
    if success and result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-union-metadata-') as directory:
    root = Path(directory)
    (root / 'Producer.rvn').write_text('''namespace UnionMetadataProbe
public union Reply<T> {
    case Item(value: T)
    case Missing
}
''')
    (root / 'Consumer.rvn').write_text('''import UnionMetadataProbe.*
import UnionMetadataProbe.Reply.*

func Create() -> Reply<int> {
    Item(42)
}

func Inspect(value: Reply<int>) -> int {
    if let Item(item) = value {
        return item
    }
    return 0
}
''')
    for name in ('Producer', 'Consumer'):
        reference = '<Reference Include="Producer"><HintPath>compiled/Producer.dll</HintPath></Reference>' if name == 'Consumer' else ''
        project = root / (name + '.rvnproj')
        project.write_text(f'''<Project>
<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
<PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
<ItemGroup><Compile Include="{name}.rvn" />{reference}</ItemGroup>
</Project>''')
        run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', project,
             '--no-project-restore', '-o', root / 'compiled'])
    assembly = root / 'compiled/Producer.dll'
    report = json.loads(run(['dotnet', bridge, '--union-metadata-report', assembly]).stdout)
    assert report == {
        'Carrier': 'UnionMetadataProbe.Reply`1',
        'Cases': [
            {'MetadataName': 'UnionMetadataProbe.Reply+Item`1', 'Name': 'Item', 'Ordinal': 0},
            {'MetadataName': 'UnionMetadataProbe.Reply+Missing', 'Name': 'Missing', 'Ordinal': 1}
        ],
        'Companions': ['UnionMetadataProbe.Reply']
    }, report
    print(json.dumps(report, indent=2))
    for mode in ('missing', 'wrong-target'):
        altered = root / (mode + '.dll')
        run(['dotnet', bridge, '--union-companion-mutation', assembly, altered, mode])
        rejected = run(['dotnet', bridge, '--union-metadata-report', altered], success=False)
        assert rejected.returncode != 0 and 'companion' in rejected.stderr.lower(), rejected.stdout + rejected.stderr
        print('Rejected companion metadata: ' + mode)
    print('Generic union metadata and separate consumer compilation passed; no neoCLR generic-union execution claimed.')
