#!/usr/bin/env python3
"""Build and qualify the integrated runtime/backend foundation (macOS/Windows)."""
import argparse
import hashlib
import importlib.util
import json
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, platform=platform.platform(), commands=[],
                  scope='workspace build, CLI selection, scalar interpreter/JIT/AOT parity, native roots and TLS adapter')

    def run(label, command):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, timeout=1800)
        (out / (label + '.stdout.log')).write_bytes(result.stdout)
        (out / (label + '.stderr.log')).write_bytes(result.stderr)
        report['commands'].append(dict(label=label, command=result.args, exitCode=result.returncode))
        if result.returncode:
            raise RuntimeError(label + ' failed; see retained logs')
        print('PASS ' + label, flush=True)
        return result.stdout

    try:
        report['revision'] = run('revision', ['git', 'rev-parse', 'HEAD']).decode().strip()
        metadata = json.loads(run('workspace', ['cargo', 'metadata', '--locked', '--format-version', '1', '--no-deps']))
        members = {p['name'] for p in metadata['packages'] if p['id'] in metadata['workspace_members']}
        if members != {'neoclr', 'neoclr-aot-poc', 'neoclr-native-tls', 'neoclr-security-provider'}:
            raise RuntimeError('Unexpected runtime workspace membership')
        report['members'] = sorted(members)
        report['inputs'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                            for p in [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', Path(__file__).resolve(),
                                      ROOT / 'tools/aot-poc/Cargo.toml', ROOT / 'tools/native-tls/Cargo.toml']}
        spec = importlib.util.spec_from_file_location('release_audit', ROOT / 'scripts/validate-release.py')
        release = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(release)
        report['noticePackages'], report['noticeFiles'] = release.audit_notices(ROOT)
        run('build', ['cargo', 'build', '--locked', '--workspace'])
        run('cli', ['cargo', 'test', '--locked', '-p', 'neoclr', '--test', 'cli', 'cli_execution_selection'])
        run('execution', ['cargo', 'test', '--locked', '-p', 'neoclr-aot-poc', '--test', 'scalar'])
        if platform.system() == 'Darwin' and platform.machine() == 'arm64':
            run('compiled-roots', ['cargo', 'test', '--locked', '-p', 'neoclr-aot-poc', '--test', 'console', 'utf8_text_slices_preserve'])
        run('native-gc', [sys.executable, ROOT / 'scripts/validate-native-gc.py', '--output', out / 'gc'])
        run('native-security', [sys.executable, ROOT / 'scripts/test-native-security.py'])
        for name in ['test_native_gc_layout.py', 'test_native_bundle_notices.py', 'test_native_https_build.py']:
            run(name, [sys.executable, '-m', 'unittest', 'discover', '-s', 'scripts', '-p', name])
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
    finally:
        (out / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
