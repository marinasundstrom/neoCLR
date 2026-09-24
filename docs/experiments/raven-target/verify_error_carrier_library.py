"""Check erased carrier layout, nested cases, source constructors, and definite assignment."""
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
        assert diagnostic in output, (command, output)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-foundation-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    names = ['Text.Utf8SliceError', 'Int32ParseError', 'IntegerDivisionError', 'Linq.SingleError', 'Storage.FileReadError', 'Storage.FileWriteError', 'ConsoleReadError']
    sources = {name: (ROOT / ('runtime/raven/src/System/' + name.replace('.', '/') + '.rvn')).read_text() for name in names}
    sample = sources['Text.Utf8SliceError']
    cases = [(name.replace('.', ''), name, sources[name], None) for name in names]
    cases += [
        ('ExtraStorage', 'Text.Utf8SliceError', sample.replace('private field Stored:', 'private field Extra: int\n    private field Stored:'), 'carrier storage'),
        ('CaseStorage', 'Text.Utf8SliceError', sample.replace('public struct OutOfRange { }', 'public struct OutOfRange { private field Extra: int }'), 'case layout'),
        ('WrongCase', 'Text.Utf8SliceError', sample.replace('OutOfRange', 'Different'), 'case layout'),
        ('WrongPayload', 'Text.Utf8SliceError', sample.replace('ValueStorage.Is<Utf8SliceError.OutOfRange>', 'ValueStorage.Is<int>'), 'Unsupported case storage payload'),
        ('WrongConstructor', 'Text.Utf8SliceError', sample.replace('ValueStorage.Pack(value)', 'ValueStorage.Pack(Utf8SliceError.InvalidBoundary())', 1), 'constructor body'),
        ('ConstructorEffect', 'Text.Utf8SliceError', sample.replace('Stored = ValueStorage.Pack(value)', 'System.Fault("unexpected")\n        Stored = ValueStorage.Pack(value)', 1), 'constructor body'),
        ('DefaultCarrier', 'Text.Utf8SliceError', sample.replace('if IsOutOfRange {', 'let invalid = default(Utf8SliceError)\n        invalid.ToString()\n        if IsOutOfRange {'), 'Read of uninitialized'),
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
        else:
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.type System.' + owner + '\n' in emitted
            assert '.field private Stored Value' in emitted
            assert 'starg this' in emitted
            assert '.custom instance System.Runtime.CompilerServices.UnionAttribute::.ctor()' in emitted
            assert '.method instance ToString() -> String' in emitted
    print(f'{len(cases)} error carrier admission cases passed')
