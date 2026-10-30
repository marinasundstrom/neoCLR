#!/usr/bin/env python3
"""Verify and execute the extracted native POC bundle without a source checkout."""
import argparse
import hashlib
import runpy
import json
from pathlib import Path
import subprocess
import sys


# The bounded retention analyzer intentionally reports unknown dynamic bodies.
# Keep this exact diagnostic visible in receipts; no other stderr is accepted.
RETENTION_WARNING = ('warning: DAM2004: dynamic-dispatch implementation bodies are not analyzed; '
                     'reflection inside them may require explicit retention\n')


def sample_output_matches(result, expected_stdout, allow_retention_warning=False):
    expected_stderr = ('', RETENTION_WARNING) if allow_retention_warning else ('',)
    return result.returncode == 0 and result.stdout == expected_stdout and result.stderr in expected_stderr


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', nargs='?', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--execution', choices=('default', 'interpreter'), help='Defaults to installed execution for JIT bundles, interpreter for older bundles')
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    args.report = args.report.resolve()
    root = args.bundle.resolve()
    if args.report.exists():
        raise FileExistsError(args.report)
    report = dict(passed=False, root=str(root), commands=[])
    try:
        manifest = json.loads((root / 'manifest.json').read_text())
        for relative, expected in manifest['files'].items():
            path = (root / relative).resolve()
            if root not in path.parents or not path.is_file():
                raise ValueError('Invalid bundle file: ' + relative)
            if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                raise ValueError('Bundle hash mismatch: ' + relative)
        report['verifiedFiles'] = len(manifest['files'])
        execution = args.execution or ('default' if manifest.get('jit', {}).get('default') else 'interpreter')
        report['execution'] = execution
        reflection = {}
        compiler = root / 'sdk/tools/rvnc/rvnc.dll'
        runtime = root / manifest['runtime']
        seed = root / manifest.get('runtimeSeed', 'lib/System.neox')
        modules = [root / name for name in manifest.get('modules', ['lib/Numbers.dll', 'lib/Http.dll'])]
        object_root = root / manifest['objectRoot'] if manifest.get('objectRoot') else None
        object_args = ['--object-root', object_root] if object_root else []
        module_args = [item for module in modules for item in ('--module', module)]
        for path in [runtime, seed, *modules, *([object_root] if object_root else [])]:
            if path.relative_to(root).as_posix() not in manifest['files']:
                raise ValueError('Untracked execution dependency: ' + str(path))

        def run(command, expected_stdout=None, allow_retention_warning=False):
            result = subprocess.run([str(p) for p in command], cwd=root, capture_output=True, text=True, encoding='utf-8', timeout=180)
            item = dict(command=[str(p) for p in command], exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
            report['commands'].append(item)
            if result.returncode or expected_stdout is not None and not sample_output_matches(result, expected_stdout, allow_retention_warning):
                raise RuntimeError('Bundle command failed: ' + json.dumps(item))
            return result

        for name, sample in manifest['samples'].items():
            folder = root / 'samples' / name
            run(['dotnet', compiler, 'neoclr', '--project', folder / 'App.rvnproj'])
            assembly = folder / 'bin/neoclr/App.dll'
            if not assembly.is_file():
                raise RuntimeError('Missing native sample output: ' + name)
            roots = sample.get('reflectionTypes', [])
            if execution == 'default' and roots:
                prepare = runpy.run_path(str(root / 'tools/run-native-sample.py'))['reflection_roots']
                reflection[name] = prepare(runtime, assembly, roots, run)[1]
            if sample.get('expected') is not None:
                context = (['--execution', 'interpreter', '--system', seed, *module_args, *object_args,
                            '--instructions', '100000000'] if execution == 'interpreter' else [])
                if name in reflection:
                    context += ['--reflection-roots', reflection[name]]
                run([runtime, 'run', assembly, *context], sample['expected'],
                    allow_retention_warning=execution == 'default')
        http_report = args.report.with_name(args.report.stem + '-http.json')
        http_options = ['--execution', execution]
        for name in ('server', 'client'):
            if 'http-json-' + name in reflection:
                http_options += ['--' + name + '-reflection-roots', reflection['http-json-' + name]]
        run([sys.executable, root / 'tools/verify-native-http-json.py', '--runtime', runtime,
             '--server', root / 'samples/http-json-server/bin/neoclr/App.dll',
             '--client', root / 'samples/http-json-client/bin/neoclr/App.dll',
             '--seed', seed, *module_args, *object_args, *http_options, '--output', http_report])
        report['http'] = json.loads(http_report.read_text())
        report['passed'] = True
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(args.report)


if __name__ == '__main__':
    main()
