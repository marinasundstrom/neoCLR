"""Check mixed constructor arguments, conversion order and single evaluation."""
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


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-constructor-arguments-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    run(['dotnet', args.bridge.resolve(), '--reference-core', root / 'demo/NeoCLR.CoreProbe.dll'])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    (root / 'Main.rvn').write_text((Path(__file__).parent / 'samples/application-constructor-arguments.rvn').read_text())
    project = root / 'Probe.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    output = root / 'imported'
    run(['dotnet', args.bridge.resolve(), '--project', project, output])
    artifact = output / 'App.neoil'
    run([args.runtime.resolve(), 'verify', artifact, '--system', system])
    result = run([args.runtime.resolve(), 'run', artifact, '--system', system])
    assert result.stdout.strip().splitlines() == [
        '1', '2', '3', 'Class arguments preserved', '42', '3',
        'Value arguments preserved', '7'], result.stdout
print('Class/value constructor conversions preserve mixed arguments and single left-to-right evaluation.')
