#!/usr/bin/env python3
"""Build Neo.Web against a verified bundle and run its framework HTTP consumer and reference app."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import threading
import urllib.request
import urllib.error
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--codegen', type=Path, help='Source-matched code generator override for the development hosting graph')
    parser.add_argument('--mode', choices=['native', 'interpreter', 'jit'], action='append')
    parser.add_argument('--component', choices=['tests', 'sample', 'sample-tests'], action='append', help='Run only changed consumers; default runs all')
    args = parser.parse_args()
    original, out = args.bundle.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = dict(passed=False, sources={str(p.relative_to(ROOT)): sha(p) for parent in
                  [ROOT / 'extensions/Neo.Web', ROOT / 'samples/web'] for p in parent.rglob('*')
                  if (p.suffix in ('.rvn', '.rvnproj') or p.name.endswith('.retention.json')) and not {'bin', 'obj'}.intersection(p.parts)}, commands=[], cases=[])
    manifest = json.loads((original / 'manifest.json').read_text())
    for relative, digest in manifest['files'].items():
        if sha(original / relative) != digest:
            raise ValueError('Bundle hash mismatch: ' + relative)
    bundle = out / 'bundle'
    shutil.copytree(original, bundle)
    lib = bundle / 'lib'
    compiler = bundle / 'sdk/tools/rvnc/rvnc.dll'
    runtime = bundle / 'bin' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
    codegen = bundle / 'bin' / ('neoclr-codegen.exe' if os.name == 'nt' else 'neoclr-codegen')
    if args.codegen:
        shutil.copy2(args.codegen.resolve(), codegen)
    report['codegenSha256'] = sha(codegen)
    report['originalManifestSha256'] = sha(original / 'manifest.json')
    report['compilerSha256'] = sha(compiler)
    env = dict(os.environ, NeoClrBundleRoot=str(bundle))

    def run(command, name, timeout=600, require_success=True):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                capture_output=True, text=True, timeout=timeout)
        (out / (name + '.log')).write_text(result.stdout + result.stderr)
        report['commands'].append(dict(name=name, command=list(map(str, command)), exitCode=result.returncode))
        if result.returncode and require_success:
            raise RuntimeError(name + ': ' + (result.stdout + result.stderr)[-10000:])
        return result

    try:
        project = ET.Element('Project', Sdk='Microsoft.NET.Sdk')
        ET.SubElement(project, 'Import', Project=str(lib / 'NeoCLR.ClassLibrary.props'))
        props = ET.SubElement(project, 'PropertyGroup')
        for key, value in dict(TargetFramework='net10.0', OutputType='Library', AssemblyName='Neo.Web',
                               EnableDefaultCompileItems='false', RavenNeoClrRuntimeAsyncHelper='').items():
            ET.SubElement(props, key).text = value
        items = ET.SubElement(project, 'ItemGroup')
        ET.SubElement(items, 'Reference', Remove='Neo.Web')
        ET.SubElement(items, 'Compile', Include=str(ROOT / 'extensions/Neo.Web/src/Neo.Web/Neo/**/*.rvn'))
        ET.indent(project)
        project_file = out / 'Neo.Web.rvnproj'
        ET.ElementTree(project).write(project_file, encoding='unicode')
        run(['dotnet', compiler, 'neoclr', '--project', project_file], 'library')
        built = out / 'bin/neoclr'
        for name in ('Neo.Web.dll', 'Neo.Web.xml', 'Neo.Web.docs'):
            path = built / name
            if path.is_dir():
                shutil.copytree(path, lib / name, dirs_exist_ok=True)
            else:
                shutil.copy2(path, lib / name)
        catalog = json.loads((lib / 'bundle.json').read_text())
        if 'Neo.Web' not in catalog['assemblyNames']:
            catalog['assemblyNames'].append('Neo.Web')
        catalog['documentation']['Neo.Web'] = dict(xml='Neo.Web.xml', markdown='Neo.Web.docs')
        props_path = lib / 'NeoCLR.ClassLibrary.props'
        tree = ET.parse(props_path)
        group = ET.SubElement(tree.getroot(), 'ItemGroup')
        if not any(r.get('Include') == 'Neo.Web' for r in tree.getroot().iter('Reference')):
            ref = ET.SubElement(group, 'Reference', Include='Neo.Web')
            ET.SubElement(ref, 'HintPath').text = '$(MSBuildThisFileDirectory)Neo.Web.dll'
        tree.write(props_path, encoding='unicode')
        for p in [lib / 'Neo.Web.dll', lib / 'Neo.Web.xml', props_path, *(lib / 'Neo.Web.docs').rglob('*')]:
            if p.is_file():
                catalog['files'][p.relative_to(lib).as_posix()] = sha(p)
        (lib / 'bundle.json').write_text(json.dumps(catalog, indent=2) + '\n')
        if 'lib/Neo.Web.dll' not in manifest['modules']:
            manifest['modules'].append('lib/Neo.Web.dll')
        manifest['files'] = {p.relative_to(bundle).as_posix(): sha(p) for p in bundle.rglob('*')
                             if p.is_file() and p != bundle / 'manifest.json'}
        (bundle / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        for name, project in [('tests', 'extensions/Neo.Web/test/Neo.Web.Tests/Neo.Web.Tests.rvnproj'),
                              ('sample', 'samples/web/src/WebDemo/WebDemo.rvnproj'),
                              ('sample-tests', 'samples/web/test/WebDemo.Tests/WebDemo.Tests.rvnproj')]:
            if args.component and name not in args.component:
                continue
            run([sys.executable, ROOT / 'scripts/build-native-project.py', '--profile',
                 'windows-http' if os.name == 'nt' else 'http', '--bundle', bundle, '--project', ROOT / project,
                 '--aot', codegen, '--output', out / name], name + '-build')
            for mode in args.mode or ['native']:
                if mode == 'native':
                    command = [str(out / name / ('app.exe' if os.name == 'nt' else 'app'))]
                else:
                    command = [str(runtime), 'run', str(out / name / 'app.dll'), '--execution', mode,
                               '--system', str(lib / 'System.runtime.neox'), '--object-root', str(lib / 'System.Runtime.dll'),
                               ]
                    if mode == 'interpreter':
                        command += ['--instructions', '100000000']
                    if mode == 'jit':
                        command += ['--jit-compiler', str(codegen), '--jit-host', str(bundle / 'bin' / ('neoclr-jit-host.dll' if os.name == 'nt' else 'neoclr-jit-host.dylib'))]
                    retention = out / name / 'app.reflection.json'
                    if mode == 'jit' and retention.exists():
                        command += ['--reflection-roots', str(retention)]
                    for module in manifest['modules']:
                        command += ['--module', str(bundle / module)]
                    build = json.loads((out / name / 'build.json').read_text())
                    for dependency in build.get('projectModules', []):
                        command += ['--module', dependency]
                    command += ['--']
                if name in ('tests', 'sample-tests'):
                    result = run(command, name + '-' + mode, timeout=600, require_success=False)
                    expected = 5 if name == 'tests' else 4
                    passed = result.returncode == 0 and f'Tests: {expected}, passed: {expected}, failed: 0, skipped: 0' in result.stdout
                    report['cases'].append(dict(name=name, mode=mode, passed=passed, stdout=result.stdout, stderr=result.stderr))
                else:
                    try:
                        case = sample(command + ['0', '120' if mode == 'interpreter' else '15'], env, mode, out)
                        case['passed'] = True
                    except Exception as error:
                        case = dict(name=name, mode=mode, passed=False, error=str(error))
                    report['cases'].append(case)
        report['passed'] = all(case['passed'] for case in report['cases'])
        if not report['passed']:
            raise RuntimeError('One or more execution checks failed; see results.json and mode logs')
    finally:
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')


def sample(command, env, mode, out):
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    lines, errors, messages = [], [], queue.Queue()
    def read_output():
        for line in process.stdout:
            lines.append(line)
            messages.put(line)
        messages.put(None)
    threading.Thread(target=read_output, daemon=True).start()
    thread = threading.Thread(target=lambda: errors.append(process.stderr.read()), daemon=True)
    thread.start()
    responses = []
    try:
        while True:
            line = messages.get(timeout=600)
            if line is None:
                raise RuntimeError('Sample exited before listening: ' + ''.join(lines + errors))
            if line.startswith('LISTENING '):
                url = line.strip().split(' ', 1)[1]
                break
        cases = [('GET', '/', 200, 'Hello, neoCLR!'),
                ('GET', '/hello/Ada', 200, 'Hello, Ada!'),
                ('GET', '/hello/current', 200, 'The current visitor is neoCLR'),
                ('POST', '/visits', 201, 'Visit 1'), ('POST', '/visits', 201, 'Visit 2'),
                ('GET', '/items/42', 200, 'The answer'), ('GET', '/items/7', 404, ''), ('GET', '/items/-1', 400, ''),
                ('GET', '/items/nope', 400, 'Bad request'),
                ('GET', '/echo/hello', 200, 'GET:hello'), ('POST', '/echo/hello', 200, 'POST:hello'),
                ('GET', '/missing/path', 404, 'No endpoint matched')]
        cases = [(*case, None, {}) for case in cases]
        cases += [
            ('GET', '/search?q=Ada+Lovelace&page=2', 200, 'Hello, Ada Lovelace! Page 2', None, {}),
            ('GET', '/search?Q=%C3%85sa&page=3', 200, 'Hello, Åsa! Page 3', None, {}),
            ('GET', '/search?q=Ada', 400, 'Bad request', None, {}),
            ('GET', '/search?q=Ada&page=bad', 400, 'Bad request', None, {}),
            ('GET', '/search?q=Ada&page=1&page=2', 400, 'Bad request', None, {}),
            ('GET', '/search?q=%FF&page=1', 400, 'Bad request', None, {}),
            ('POST', '/greetings', 201, 'Hello, Grace!', b'{"Name":"Grace"}', {'Content-Type': 'application/json'}),
            ('POST', '/greetings', 201, 'Hello, Ada!', b'{"name":"Ada","unused":42}', {'Content-Type': 'application/vnd.demo+json; charset=utf-8'}),
            ('POST', '/greetings', 400, 'Bad request', b'{', {'Content-Type': 'application/json'}),
            ('POST', '/greetings', 415, 'Unsupported media type', b'Grace', {'Content-Type': 'text/plain'}),
            ('POST', '/json-text', 200, 'plain value', b'"plain value"', {'Content-Type': 'application/json'}),
            ('POST', '/json-text', 400, 'Bad request', b'42', {'Content-Type': 'application/json'}),
        ]
        computed = 0
        for index in range(5000):
            computed = (computed * 17 + index) % 1009
        cached_body = 'Computed ' + str(computed)
        cases += [
            ('GET', '/cached', 200, cached_body, None, {}),
            ('GET', '/cached', 200, cached_body, None, {}),
            ('GET', '/cache-stats', 200, 'hits=1;misses=1;executions=1', None, {}),
            ('GET', '/uncached', 200, cached_body, None, {}),
            ('GET', '/cached', 200, cached_body, None, {'Cache-Control': 'no-cache'}),
            ('GET', '/cache-stats', 200, 'hits=1;misses=1;executions=3', None, {}),
        ]
        for method, path, status, body, data, headers in cases:
            try:
                response = urllib.request.urlopen(urllib.request.Request(url + path, method=method, data=data, headers=headers), timeout=60)
            except urllib.error.HTTPError as error:
                response = error
            with response:
                actual = response.read().decode('utf-8')
                if response.status != status or actual != body:
                    raise RuntimeError('Reference app response mismatch: ' + actual)
                responses.append(dict(method=method, path=path, status=status, body=actual))
        code = process.wait(timeout=180)
        thread.join(timeout=5)
        if code or 'STOPPED\n' not in lines:
            raise RuntimeError('Reference app did not stop cleanly: ' + ''.join(lines + errors))
        return dict(name='sample', mode=mode, exitCode=code, stdout=''.join(lines), stderr=''.join(errors), responses=responses)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        (out / ('sample-' + mode + '.log')).write_text(''.join(lines + errors))


if __name__ == '__main__':
    main()
