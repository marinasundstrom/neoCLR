"""Check standard error union admission and reject mismatched case contracts."""
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
        ('ExtraStorage', 'Text.Utf8SliceError', sample.replace('public union Utf8SliceError {', 'public union Utf8SliceError {\n    private var extra: int'), 'standard union library shape'),
        ('CaseStorage', 'Text.Utf8SliceError', sample.replace('case OutOfRange', 'case OutOfRange(value: int)').replace('OutOfRange =>', 'OutOfRange(_) =>'), 'standard union library shape'),
        ('WrongCase', 'Text.Utf8SliceError', sample.replace('OutOfRange', 'Different'), 'case metadata does not match'),
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
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'],
            'RAV2115' if name == 'ExtraStorage' else None)
        if name == 'ExtraStorage':
            assert not (folder / 'bin' / (name + '.dll')).exists()
            continue
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.' + owner, output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.type System.' + owner + '\n' in emitted
            assert '.field private Stored Value' not in emitted
            assert '.implements System.Runtime.CompilerServices.IUnion' in emitted
            assert '.custom instance System.Runtime.CompilerServices.UnionAttribute::.ctor()' in emitted
            assert '.method instance override byref ToString() -> String' in emitted
    print(f'{len(cases)} error carrier admission cases passed')
