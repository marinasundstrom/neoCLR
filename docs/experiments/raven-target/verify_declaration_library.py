"""Check memberless value declarations without silently dropping implementation behavior."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-declaration-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = 'namespace System\npublic struct RuntimeTypeHandle { }'
    for name, text, diagnostic in [
        ('Valid', source, None),
        ('Storage', source.replace('{ }', '{ private field payload: int }'), 'value library layout'),
        ('Member', source.replace('{ }', '{ func Surprise() -> int { return 1 } }'), 'does not match reference contract'),
        ('Constructor', source.replace('{ }', '{ public init() { System.Fault("unexpected") } }'), 'does not match reference contract'),
        ('Category', source.replace('struct', 'class'), 'value/reference representation'),
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
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'), core, 'System.RuntimeTypeHandle', output], diagnostic is None, diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            result = (output / 'Implementation.neoil').read_text()
            assert '.type System.RuntimeTypeHandle\n.end' in result
            assert '.field' not in result
            assert not re.search(r'(?m)^\.method', result)
    print('Memberless declaration admitted; extra storage, members, constructor behavior and wrong category rejected.')
