#!/usr/bin/env python3
"""Check JsonArray terminal bounds errors through concrete and Sequence access."""
import argparse
import hashlib
import json
import platform
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--build', required=True, type=Path)
    parser.add_argument('--runtime', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    bundle, build, runtime = args.bundle.resolve(), args.build.resolve(), args.runtime.resolve()
    library = bundle / 'lib'
    executable = build / ('app.exe' if platform.system() == 'Windows' else 'app')
    interpreted = [runtime, 'run', build / 'app.dll', '--system', library / 'System.runtime.neox']
    for name in ('System.Runtime', 'System.Data', 'System.Networking', 'System.Web'):
        interpreted += ['--module', library / (name + '.dll')]
    interpreted += ['--object-root', library / 'System.Runtime.dll', '--']
    inputs = [Path(__file__), Path(__file__).with_name('Bounds.rvn'), Path(__file__).with_name('Bounds.rvnproj'),
              ROOT / 'runtime/raven/src/System/Data/Json/JsonValue.rvn']
    report = dict(passed=False, platform=platform.platform(), cases=[],
                  inputs={str(p.relative_to(ROOT)): sha(p) for p in inputs},
                  artifacts={str(p): sha(p) for p in (bundle / 'manifest.json', runtime, executable, build / 'app.dll')})
    for mode, command in (('native', [executable]), ('interpreter', interpreted)):
        for boundary in ('negative', 'end'):
            for view in ('concrete', 'sequence'):
                arguments = [boundary] + (['sequence'] if view == 'sequence' else [])
                result = subprocess.run(list(map(str, [*command, *arguments])), capture_output=True, text=True, timeout=30)
                passed = result.returncode != 0 and not result.stdout and 'JsonArray index out of range' in result.stderr
                report['cases'].append(dict(mode=mode, boundary=boundary, view=view, passed=passed,
                                            command=result.args, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    report['passed'] = all(case['passed'] for case in report['cases'])
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print('JSON sequence bounds:', 'PASS' if report['passed'] else 'FAIL')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
