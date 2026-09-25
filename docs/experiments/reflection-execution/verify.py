"""Compile and execute the public reflection consumer against a matching local bundle."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    parser.add_argument('--runner', type=Path, required=True)
    args = parser.parse_args()
    bundle = args.toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    with tempfile.TemporaryDirectory(prefix='neoclr-reflection-consumer-') as folder:
        root = Path(folder)
        for name in ('Main.rvn', 'ReflectionExecution.rvnproj'):
            shutil.copyfile(here / name, root / name)
        built = subprocess.run(['dotnet', 'msbuild', str(root / 'ReflectionExecution.rvnproj'),
                                '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
        assert built.returncode == 0, built.stdout + built.stderr
        assert 'warning ' not in built.stdout, built.stdout
        result = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                                 str(bundle / 'lib/System.neoil'), '512', '100000000'],
                                capture_output=True, text=True, timeout=120)
        assert result.returncode == 0, result.stdout + result.stderr
        assert result.stdout.splitlines() == ['Reflection checks passed'], result.stdout
        assert 'live=0' in result.stderr, result.stderr
        print(result.stdout + result.stderr)
        # A non-void method ending in Fault must import without fallthrough and
        # preserve the user's message when executed.
        shutil.copyfile(here / 'TerminalFault.rvn', root / 'Main.rvn')
        terminal_build = subprocess.run(['dotnet', 'msbuild', str(root / 'ReflectionExecution.rvnproj'),
                                         '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=240)
        assert terminal_build.returncode == 0, terminal_build.stdout + terminal_build.stderr
        terminal = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                                   str(bundle / 'lib/System.neoil'), '512', '100000000'],
                                  capture_output=True, text=True, timeout=120)
        assert terminal.returncode != 0, terminal.stdout + terminal.stderr
        assert 'Missing reflection property' in terminal.stderr, terminal.stderr
        assert 'System.Fault returned unexpectedly' not in terminal.stderr, terminal.stderr
        print('Terminal Fault import and message check passed')


if __name__ == '__main__':
    main()
