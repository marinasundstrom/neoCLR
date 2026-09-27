"""Compile the public GC class and execute its contract consumer."""
import argparse
from pathlib import Path
import subprocess
import sys
import shutil
import tempfile
root = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(root / 'docs/experiments/raven-target'))
from collection_library import build
parser = argparse.ArgumentParser(description=__doc__)
for name in ('bridge', 'runtime', 'runner'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
bridge, runtime, runner = (p.resolve() for p in (args.bridge,args.runtime,args.runner))
with tempfile.TemporaryDirectory(prefix='neoclr-runtime-gc-') as directory:
    work=Path(directory)
    (work/'demo').mkdir()
    subprocess.run(['dotnet',str(bridge),'--reference-core',str(work/'demo/NeoCLR.CoreProbe.dll')],check=True)
    (work/'System.neoil').write_text(build(root/'runtime/System.neoil'))
    (work/'Test.rvnproj').write_text(f'<Project><PropertyGroup><NeoCLRRoot>{work}</NeoCLRRoot></PropertyGroup><Import Project="{root}/build/NeoCLR.Raven.props"/><ItemGroup><Compile Include="Main.rvn"/></ItemGroup></Project>')
    for source, expected in [
        (Path(__file__).with_name('Main.rvn'), 'GC counters, collection and retained references passed'),
        (Path(__file__).with_name('Async.rvn'), 'GC preserves queued callbacks and async entry state'),
    ]:
        (work/'Main.rvn').write_text(source.read_text())
        subprocess.run(['dotnet',str(bridge),'--project',str(work/'Test.rvnproj'),str(work/source.stem)],check=True)
        app=work/source.stem/'App.neoil'
        evidence=root/'target/runtime-gc'/source.stem
        evidence.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(app,evidence/'App.neoil')
        shutil.copyfile(work/'System.neoil',evidence/'System.neoil')
        subprocess.run([str(runtime),'verify',str(app),'--system',str(work/'System.neoil')],check=True)
        result=subprocess.run([str(runner),str(app),str(work/'System.neoil'),'512','1000000'],text=True,capture_output=True,timeout=180)
        print(result.stdout,end=''); print(result.stderr,end=''); result.check_returncode()
        assert expected in result.stdout
