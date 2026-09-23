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
    parser.add_argument('--consumer-root', type=Path, default=HERE, help='Alternate controlled consumer using the same experimental notification adapter')
    parser.add_argument('--adapter-source', type=Path, default=HERE / 'Workers.rvn', help='Isolated bootstrap adapter to substitute for this check')
    args = parser.parse_args()
    consumer = args.consumer_root.resolve()
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
  <ItemGroup><Compile Include="{escape(str(args.adapter_source.resolve()))}" /></ItemGroup>
</Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', root / 'compiled'])
        run(['dotnet', bridge, '--library-implementation', root / 'compiled/Workers.dll', core,
             'System.Concurrency.Thread', root / 'imported'])
        generated = fragments((root / 'imported/Implementation.neoil').read_text(), 'Workers', 'System.Concurrency.Thread')
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
            shutil.copyfile(consumer / name, root / name)
        env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
        run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
        result = run([bundle / 'bin/neoclr', 'run', root / 'bin/neoclr/Debug/App.neoil',
                      '--system', root / 'System.neoil', '--gc-stats'])
        expected = (consumer / 'expected.txt').read_text()
        if consumer == HERE:
            assert result.stdout == expected, result.stdout + result.stderr
        else:
            # Independent host jobs may finish in any order; each outcome appears once.
            assert sorted(result.stdout.splitlines()) == sorted(expected.splitlines()), result.stdout
            assert result.stdout.splitlines()[:2] == expected.splitlines()[:2], result.stdout
            live = re.search(r'live=(\d+)', result.stderr)
            assert live and int(live[1]) == 0, result.stderr
        collections = re.search(r'collections=(\d+)', result.stderr)
        assert collections and int(collections[1]) > 0, result.stderr
        print(result.stdout, end='')
        print(result.stderr, end='')
        print('Isolated library adapter, actual VM completion, await and GC: passed')
        if consumer != HERE:
            if (consumer / 'Fault.rvn').exists():
                shutil.copyfile(consumer / 'Fault.rvn', root / 'Main.rvn')
                run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
                failed = subprocess.run([str(bundle / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'),
                                         '--system', str(root / 'System.neoil')], capture_output=True, text=True, timeout=120)
                assert failed.returncode != 0 and 'code=UserFault' in failed.stderr, failed.stdout + failed.stderr
                assert 'Unexpected continuation' not in failed.stdout, failed.stdout
                print('Producer UserFault remains a Fault, not Task cancellation')
            if (consumer / 'Affinity.rvn').exists():
                shutil.copyfile(consumer / 'Affinity.rvn', root / 'Main.rvn')
                run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
                affinity = run([bundle / 'bin/neoclr', 'run', root / 'bin/neoclr/Debug/App.neoil',
                                '--system', root / 'System.neoil'])
                assert affinity.stdout == (consumer / 'affinity.expected.txt').read_text(), affinity.stdout + affinity.stderr
                print(affinity.stdout, end='')
                print('Current producer-bound await and caller-bound result observation: characterized')
            if (consumer / 'Forbidden.rvn').exists():
                shutil.copyfile(consumer / 'Forbidden.rvn', root / 'Main.rvn')
                rejected = subprocess.run(['dotnet', 'msbuild', str(root / 'DelayedCopy.rvnproj'), '-nologo', '-v:minimal'],
                                          env=env, capture_output=True, text=True, timeout=120)
                diagnostics = rejected.stdout + rejected.stderr
                assert rejected.returncode != 0 and "error RAV0234" in diagnostics and "'Runtime' was not found" in diagnostics, diagnostics
                print('Normal reference core rejects bootstrap runtime-service calls')
            return
        shutil.copyfile(HERE / 'Busy.rvn', root / 'Main.rvn')
        run(['dotnet', 'msbuild', root / 'DelayedCopy.rvnproj', '-nologo', '-v:minimal'], env=env)
        busy = run([bundle / 'bin/neoclr', 'run', root / 'bin/neoclr/Debug/App.neoil',
                    '--system', root / 'System.neoil'])
        assert busy.stdout == (HERE / 'busy.expected.txt').read_text(), busy.stdout + busy.stderr
        print('Completion during continuously reposted guest work: passed')


if __name__ == '__main__':
    main()
