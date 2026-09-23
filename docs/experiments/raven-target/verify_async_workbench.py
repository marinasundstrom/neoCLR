"""Build saved async samples using only an extracted neoCLR bundle and Raven SDK."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as ET

CASES = {
    'library-async-default-queue': 'Hello on a worker\n',
    'library-workers': 'Hello, thread\nHello, pool\n',
    'library-task-producer': '42\n',
    'library-task-composition': '42\n',
    'library-task-result': '42\n',
    'library-task-propagation': 'Unavailable\n',
    'library-async-cancellation': 'Cancelled\n',
    'library-async': 'Suspended\n42\n',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--sdk', type=Path, required=True)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    bundle, sdk = args.bundle.resolve(), args.sdk.resolve()
    report = {'bundle': str(bundle), 'sdk': str(sdk), 'cases': [], 'passed': False}
    try:
        with tempfile.TemporaryDirectory(prefix='neoclr-async-workbench-') as directory:
            root = Path(directory)
            project = root / 'Demo.rvnproj'
            tree = ET.parse(bundle / 'msbuild-demo/Demo.rvnproj')
            properties = tree.getroot().find('PropertyGroup')
            # The compiler reloads the saved project in a separate process; MSBuild
            # command-line properties alone do not configure that second load.
            properties.find('NeoCLRRoot').text = str(bundle)
            ET.SubElement(properties, 'RavenSdkRoot').text = str(sdk)
            tree.write(project, encoding='unicode')
            for name, expected in CASES.items():
                shutil.copyfile(bundle / 'tools/samples' / (name + '.rvn'), root / 'Main.rvn')
                built = subprocess.run([
                    'dotnet', 'msbuild', str(project), '-nologo', '-v:minimal',
                    '-p:NeoCLRRoot=' + str(bundle), '-p:RavenSdkRoot=' + str(sdk),
                ], cwd=root, capture_output=True, text=True, timeout=180)
                if built.returncode:
                    raise RuntimeError(name + ': build failed\n' + built.stdout + built.stderr)
                output = root / 'bin/neoclr/Debug'
                ran = subprocess.run([
                    str(bundle / 'bin/neoclr'), 'run', str(output / 'App.neoil'),
                    '--system', str(output / 'System.neoil'),
                ], cwd=root, capture_output=True, text=True, timeout=60)
                if ran.returncode or ran.stdout != expected:
                    raise RuntimeError(name + ': unexpected result\n' + ran.stdout + ran.stderr)
                report['cases'].append({'sample': name, 'output': ran.stdout, 'passed': True})
                print(name + ': Passed', flush=True)
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        if args.report:
            args.report.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
