"""Validate checked Raven interface authoring using the existing Clock contract."""
import argparse
from pathlib import Path
import subprocess
import tempfile
import time
from datetime import datetime
from xml.sax.saxutils import escape
from build_runtime_library import ROOT
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()


def run(command, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    output = result.stdout + result.stderr
    assert (result.returncode == 0) == (diagnostic is None), output
    if diagnostic:
        assert diagnostic in output, output
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-interface-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Clock.rvn').read_text()
    unsupported = 'Unsupported library interface contract'
    mismatch = 'Library interface does not match reference contract'
    cases = [
        ('Valid', source, None),
        ('Class', 'namespace System\npublic class Clock {}', unsupported),
        ('Generic', source.replace('interface Clock', 'interface Clock<T>'), unsupported),
        ('Inherited', source.replace('interface Clock {', 'interface Clock : Extra {') + '\npublic interface Extra {}', unsupported),
        ('Extra', source.replace('val Now:', 'func Extra() -> int\n    val Now:'), mismatch),
        ('ReturnType', source.replace('Now: Instant', 'Now: string'), mismatch),
        ('Property', source.replace('val Now: Instant { get }', 'func get_Now() -> Instant'), mismatch),
        ('ForeignType', source + '\npublic class Instant {}', mismatch),
    ]
    for name, text, diagnostic in cases:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'])
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.Clock', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            result = (output / 'Implementation.neoil').read_text()
            assert '.interface System.Clock' in result
            assert '.property instance Now() -> System.Instant' in result
            assert '.method instance get_Now() -> System.Instant' in result
            assert '.method instance get_Now() -> System.Instant\n.end' in result

    # Exercise both a user implementation and SystemClock through the regenerated
    # runtime-library contract, not only the reference metadata used for compilation.
    run(['dotnet', args.bridge.resolve(), '--reference-core', core])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    project = root / 'Demo.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    for name in ('library-instants', 'library-clock'):
        (root / 'Main.rvn').write_text((Path(__file__).parent / 'samples' / (name + '.rvn')).read_text())
        output = root / name
        run(['dotnet', args.bridge.resolve(), '--project', project, output])
        artifact = output / 'App.neoil'
        run([args.runtime.resolve(), 'verify', artifact, '--system', system])
        before = time.time()
        result = run([args.runtime.resolve(), 'run', artifact, '--system', system])
        after = time.time()
        if name == 'library-instants':
            assert result.stdout.strip().splitlines() == ['0', '-1', '0', '-1', 'Same duration', 'System clock'], result.stdout
        else:
            parts = [int(value) for value in result.stdout.strip().splitlines()]
            assert len(parts) == 6, result.stdout
            observed = datetime(*parts)
            # Accept either occurrence of a repeated civil time at a DST transition.
            assert any(before - 2 <= observed.replace(fold=fold).timestamp() <= after + 2
                       for fold in (0, 1)), result.stdout
print('Clock interface imports; class/generic/inherited shapes, extra members, property/return changes and foreign type identity rejected.')
print('FixedClock and SystemClock consumers execute; local-clock output matches the host.')
