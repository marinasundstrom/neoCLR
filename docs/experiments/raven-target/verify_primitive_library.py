"""Check primitive wrapper admission without granting intrinsic storage to guests."""
import argparse
import re
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()


def run(command, success=True, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    if diagnostic:
        assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-primitive-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Int64.rvn').read_text()
    for name, text, diagnostic in [
        ('Valid', source, None),
        ('WrongStorage', source.replace('m_value: long', 'm_value: int'), 'Unsupported primitive library storage'),
        ('ExtraStorage', source.replace('private field m_value: long', 'private field m_value: long\n    private field extra: int'), 'Unsupported primitive library storage'),
        ('Mutation', source.replace('if m_value < other', 'm_value = 0L\n        if m_value < other'), 'Primitive library backing storage is readonly'),
        ('Constructor', source.replace('    func CompareTo', '    public init() { System.Fault("unexpected") }\n\n    func CompareTo'), 'Unsupported primitive library storage'),
    ]:
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
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'), core, 'System.Int64', output], diagnostic is None, diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            result = (output / 'Implementation.neoil').read_text()
            assert '.field' not in result and 'ldobj Int64' in result
            assert '.method instance readonly byref CompareTo(Int64 other)' in result
            assert not re.search(r'(?m)^\.method[^\n]*\.ctor', result)
    print('Primitive wrapper admitted; wrong/extra storage, writes and effectful constructors rejected.')
