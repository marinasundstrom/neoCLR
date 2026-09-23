"""Build an isolated notification adapter and run the GC/await consumer."""
import argparse
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import tempfile
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def run(command, **kwargs):
    result = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=120, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    compiler = bundle / 'raven-sdk/tools/rvnc/rvnc.dll'
    bridge = bundle / 'tools/bridge/Probe.dll'
    fragments = runpy.run_path(str(ROOT / 'docs/experiments/raven-target/build_runtime_library.py'))['fragments']
    with tempfile.TemporaryDirectory(prefix='neoclr-delayed-copy-') as folder:
        root = Path(folder)
        (root / 'demo').mkdir()
        core = root / 'demo/NeoCLR.CoreProbe.dll'
        run(['dotnet', bridge, '--reference-library-core', core])
        project = root / 'Workers.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>Workers</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="{escape(str(HERE / 'Workers.rvn'))}" /></ItemGroup>
</Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
        run(['dotnet', bridge, '--library-implementation', root / 'compiled/Workers.dll', core,
             'System.Threading.Thread', root / 'imported'])
        generated = fragments((root / 'imported/Implementation.neoil').read_text(), 'Workers', 'System.Threading.Thread')
        system = (bundle / 'lib/System.neoil').read_text()
        for name, replacement in generated.items():
            original = (ROOT / 'runtime/raven/generated' / name).read_text()
            if system.count(original) != 1:
                raise AssertionError('Expected matching development worker implementation: ' + name)
            system = system.replace(original, replacement)
        (root / 'System.neoil').write_text(system)
        # Normal application references exclude bootstrap host services.
        run(['dotnet', bridge, '--reference-core', core])
        for name in ['Copy.rvn', 'Main.rvn', 'DelayedCopy.rvnproj']:
            shutil.copyfile(HERE / name, root / name)
        env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
        run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
        result = run([bundle / 'bin/neoclr', 'run', root / 'bin/neoclr/Debug/App.neoil',
                      '--system', root / 'System.neoil', '--gc-stats'])
        assert result.stdout == (HERE / 'expected.txt').read_text(), result.stdout + result.stderr
        collections = re.search(r'collections=(\d+)', result.stderr)
        assert collections and int(collections[1]) > 0, result.stderr
        print(result.stdout, end='')
        print(result.stderr, end='')
        print('Isolated library adapter, actual VM completion, await and GC: passed')
        shutil.copyfile(HERE / 'Busy.rvn', root / 'Main.rvn')
        run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
        busy = run([bundle / 'bin/neoclr', 'run', root / 'bin/neoclr/Debug/App.neoil',
                    '--system', root / 'System.neoil'])
        assert busy.stdout == (HERE / 'busy.expected.txt').read_text(), busy.stdout + busy.stderr
        print('Completion during continuously reposted guest work: passed')


if __name__ == '__main__':
    main()
