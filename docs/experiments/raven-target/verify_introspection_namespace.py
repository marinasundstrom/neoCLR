"""Compile and execute migrated introspection consumers; reject the old namespace."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from collection_library import ROOT, build

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bridge', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = ['dotnet', str(args.bridge.resolve())]
runtime = str(args.runtime.resolve())
samples = Path(__file__).resolve().parent / 'samples'


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-introspection-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    run([*bridge, '--reference-core', root / 'demo/NeoCLR.CoreProbe.dll'])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    project = root / 'Demo.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    for name in ('library-reflection', 'library-flags', 'library-type-preview'):
        (root / 'Main.rvn').write_text((samples / (name + '.rvn')).read_text())
        output = root / name
        run([*bridge, '--project', project, output])
        artifact = output / 'App.neoil'
        run([runtime, 'verify', artifact, '--system', system])
        result = run([runtime, 'run', artifact, '--system', system])
        if name == 'library-reflection':
            expected = (samples / 'library-reflection.expected.txt').read_text()
            assert result.stdout.strip() == expected.strip(), result.stdout
    (root / 'Main.rvn').write_text('''import System.*
func Main() {
    let info: System.Reflection.TypeInfo = typeof(int).Info
}
''')
    rejected = run([*bridge, '--project', project, root / 'old-namespace'], False)
    assert 'Reflection' in rejected.stdout + rejected.stderr
    assert not (root / 'old-namespace/App.neoil').exists()
print('Three introspection consumers execute; old descriptor namespace rejected.')
