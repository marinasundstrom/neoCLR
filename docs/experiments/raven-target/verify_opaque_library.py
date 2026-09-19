"""Validate opaque storage, receiver ABI, signatures, and rejected allocation/mutation."""
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
    strings = (ROOT / 'runtime/raven/src/System/String.rvn').read_text()
    cases = [
        ('String', 'String', strings, None),
        ('StringStorage', 'String', strings.replace('m_value', 'other_storage'), 'String storage'),
        ('StringExtraStorage', 'String', strings.replace('private field m_value:', 'private field extra: int\n    private field m_value:'), 'String storage'),
        ('StringWrite', 'String', strings.replace('return RuntimeServices.StringByteCount(m_value)', 'm_value = "changed"\n        return 0'), 'backing storage is readonly'),
        ('StringConstruct', 'String', strings.replace('return RuntimeServices.StringByteCount(m_value)', 'let value = String()\n        return 0'), 'requires a runtime factory'),
        ('StringSignature', 'String', strings.replace('Concat(value0:', 'Concat(renamed:').replace('StringConcat(value0,', 'StringConcat(renamed,'), 'export does not match'),
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
        elif owner == 'String':
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.type System.String\n' in emitted
            assert '.method instance readonly byref Equals(String other)' in emitted
            assert '.method instance GetUtf8ByteCount()' in emitted
            assert '.field ' not in emitted
    print(f'{len(cases)} opaque library admission cases passed')
