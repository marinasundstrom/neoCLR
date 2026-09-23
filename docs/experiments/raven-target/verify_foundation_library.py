"""Validate generic interface contracts and the nested local-calendar layout."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()


def run(command, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    output = result.stdout + result.stderr
    assert (result.returncode == 0) == (diagnostic is None), output
    if diagnostic:
        assert diagnostic in output, output
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-foundation-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Equatable.rvn').read_text()
    local = (ROOT / 'runtime/raven/src/System/LocalDateTime.rvn').read_text()
    file_source = (ROOT / 'runtime/raven/src/System/Storage/FileText.rvn').read_text()
    mapping = (ROOT / 'runtime/raven/src/System/Collections/Map.rvn').read_text()
    mutable = (ROOT / 'runtime/raven/src/System/Collections/MutableMap.rvn').read_text()
    mismatch = 'Library interface does not match reference contract'
    cases = [
        ('Equality', 'Equatable', source, None),
        ('WrongArgument', 'Equatable', source.replace('other: T', 'other: int'), mismatch),
        ('WrongName', 'Equatable', source.replace('other: T', 'renamed: T'), mismatch),
        ('WrongResult', 'Equatable', source.replace('-> bool', '-> int'), mismatch),
        ('WrongArity', 'Equatable', source.replace('Equatable<T>', 'Equatable<T, U>'), mismatch),
        ('WrongParameter', 'Equatable', source.replace('Equatable<T>', 'Equatable<T, U>').replace('other: T', 'other: U'), mismatch),
        ('Map', 'Collections.Map', mapping, None),
        ('SwappedMapParameter', 'Collections.Map', mapping.replace('Find(key: K)', 'Find(key: V)'), mismatch),
        ('MutableMap', 'Collections.MutableMap', mutable, None),
        ('MissingParent', 'Collections.MutableMap', mutable.replace(' : Map<K, V>', ''), mismatch),
        ('WrongParent', 'Collections.MutableMap', mutable.replace(' : Map<K, V>', ' : Map<V, K>'), mismatch),
        ('DefaultNativePayload', 'Storage.FileText', file_source.replace('RuntimeServices.ReadAllText(path, maxBytes)', 'default(System.Value)'), 'Read of uninitialized'),
        ('UnsupportedPayload', 'Storage.FileText', file_source.replace('UnpackValue<byte>', 'UnpackValue<long>'), 'Unsupported erased native payload type'),
        ('Local', 'LocalDateTime', local, None),
        ('Reordered', 'LocalDateTime', local.replace('private field StoredDate: Date\n    private field StoredTime: Time', 'private field StoredTime: Time\n    private field StoredDate: Date'), 'value library layout'),
        ('ExtraStorage', 'LocalDateTime', local.replace('private field StoredDate:', 'private field Extra: int = 0\n    private field StoredDate:'), 'value library layout'),
        ('PublicFactory', 'LocalDateTime', local.replace('internal static func', 'public static func'), 'Internal library factory'),
    ]
    for name, owner, text, diagnostic in cases:
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
             core, 'System.' + owner, output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        elif owner == 'LocalDateTime':
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.method internal static FromUnixTimeTicks(Int64 ticks)' in emitted
            assert '.field private StoredDate System.Date\n.field private StoredTime System.Time' in emitted
        elif owner == 'Equatable':
            assert '.interface System.Equatable<T0>' in (output / 'Implementation.neoil').read_text()
print('Generic contracts reject changed arity, type parameter, argument name/type and return type.')
print('LocalDateTime preserves nested field order and internal factory visibility; malformed layouts rejected.')
