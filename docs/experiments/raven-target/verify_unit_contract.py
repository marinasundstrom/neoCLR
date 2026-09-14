"""Verify the selected Void contract through Raven project compilation and neoCLR execution."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape

from build_runtime_library import ROOT
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, name).resolve() for name in ('compiler', 'bridge', 'runtime'))

def run(command):
    result = subprocess.run([str(value) for value in command], capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout

with tempfile.TemporaryDirectory(prefix='neoclr-unit-contract-') as temporary:
    root = Path(temporary)
    demo = root / 'demo'
    demo.mkdir()
    core = demo / 'NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--reference-core', core])
    (demo / 'Main.rvn').write_text('''import System.Void
import System.OverflowError
import System.Result.*
import System.Console.*

func Notify() {
    WriteLine("notified")
}

func Take(value: Void) -> int {
    return 42
}

func Complete(fail: bool) -> System.Result<Void, OverflowError> {
    if fail {
        return Error(OverflowError())
    }
    return Ok(())
}

func Forward(fail: bool) -> System.Result<Void, OverflowError> {
    Complete(fail)?
    return Ok(())
}

func Main() {
    Notify()
    let value = ()
    WriteLine(Take(value))
    WriteLine(Take(Notify()))
    match Forward(false) {
        Ok(_) => WriteLine("complete")
        Error(_) => WriteLine("unexpected error")
    }
    match Forward(true) {
        Ok(_) => WriteLine("unexpected success")
        Error(_) => WriteLine("error")
    }
}
''')
    project = demo / 'Probe.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <AssemblyName>UnitContractConsumer</AssemblyName>
    <NeoCLRRoot>{escape(str(root))}</NeoCLRRoot>
  </PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
    image = root / 'compiled/UnitContractConsumer.dll'
    metadata = json.loads(run(['dotnet', bridge, '--unit-contract-check', image]))
    imported = root / 'imported'
    run(['dotnet', bridge, '--import', image, core, imported])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', imported / 'App.neoil', '--system', system])
    output = run([runtime, 'run', imported / 'App.neoil', '--system', system])
    assert output.splitlines() == ['notified', '42', 'notified', '42', 'complete', 'error'], output
    print(json.dumps({'metadata': metadata, 'executedOutput': output.splitlines(),
                      'literalAndCallValues': True, 'resultVoidPropagation': True}, indent=2))
