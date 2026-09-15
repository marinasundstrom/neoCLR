"""Verify the terminal namespace fault API through a separately compiled Raven consumer."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT, check_snapshot
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, n).resolve() for n in ('compiler', 'bridge', 'runtime'))


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


check_snapshot()
with tempfile.TemporaryDirectory(prefix='neoclr-fault-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--reference-core', core])
    for spelling in ('System.Fault', 'Fault'):
        source = '\n'.join([
            'import System.*',
            'import System.Console.*',
            'func Message(prefix: string) -> string {',
            '    return prefix + " has no current element: åäö"',
            '}',
            'func Main() {',
            '    let message = Message("Query iterator")',
            f'    {spelling}(message)',
            '    WriteLine("UNREACHABLE")',
            '}',
        ])
        (root / 'Main.rvn').write_text(source)
        project = root / 'Demo.rvnproj'
        project.write_text(f'''<Project><PropertyGroup><OutputType>Exe</OutputType>
    <AssemblyName>Consumer</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
    <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
    <ItemGroup><Compile Include="Main.rvn" /></ItemGroup></Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
        imported = root / ('imported-' + spelling)
        run(['dotnet', bridge, '--import', root / 'compiled/Consumer.dll', core, imported])
        system = root / 'System.neoil'
        system.write_text(build(ROOT / 'runtime/System.neoil'))
        run([runtime, 'verify', imported / 'App.neoil', '--system', system])
        result = subprocess.run([str(runtime), 'run', str(imported / 'App.neoil'), '--system', str(system)], capture_output=True, text=True, timeout=120)
        assert result.returncode != 0, result
        assert 'Query iterator has no current element: åäö' in result.stderr, result.stderr
        assert 'UNREACHABLE' not in result.stdout, result.stdout
        print('Raven namespace fault function reports a computed message and terminates the guest')
