#!/usr/bin/env python3
"""Compile existing Raven async consumers and compare native context policies."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--build', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[4]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads(args.build.read_text())
assert build['passed']
recipes = {entry['phase']: entry['command'] for entry in build['commands']}
bundle = Path(recipes['source-compilation'][1]).parents[3]
env = dict(os.environ, NeoClrBundleRoot=str(bundle))
report = dict(passed=False, commands=[], inputs={}, cases=[])


def run(command, label, expected=0):
    result = subprocess.run(command, cwd=root, env=env, capture_output=True,
                            text=True, timeout=180)
    log = out / (label + '.stdout')
    log.write_text(result.stdout)
    report['commands'].append(dict(command=command, exitCode=result.returncode,
                                   stdoutFile=str(log),
                                   stdoutSha256=hashlib.sha256(log.read_bytes()).hexdigest(),
                                   stderr=result.stderr,
                                   stdout=result.stdout if len(command) == 1 else None))
    assert result.returncode == expected, (label, result.returncode, result.stderr, result.stdout[-4000:])
    return result


try:
    for name in ['RuntimeAwait', 'RuntimeUse', 'RuntimeStackReuse',
                 'RuntimeNativeContract', 'RuntimeAwaitFault']:
        source = root / 'docs/experiments/runtime-async' / (name + '.rvn')
        report['inputs'][str(source)] = hashlib.sha256(source.read_bytes()).hexdigest()
        directory = out / name
        directory.mkdir()
        project = directory / (name + '.rvnproj')
        project.write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="{escape(str(bundle / 'lib/NeoCLR.ClassLibrary.props'))}" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
    <AssemblyName>{name}</AssemblyName><EnableDefaultCompileItems>false</EnableDefaultCompileItems>
  </PropertyGroup>
  <ItemGroup><Compile Include="{escape(str(source))}" /></ItemGroup>
</Project>
''')
        command = recipes['source-compilation'][:]
        command[-1] = str(project)
        run(command, name + '-source')
        assemblies = list(directory.rglob(name + '.dll'))
        assert len(assemblies) == 1, assemblies
        obj = directory / 'app.o'
        command = recipes['native-compilation'][:]
        old_object = command[4]
        command[2], command[4] = str(assemblies[0]), str(obj)
        run(command, name + '-codegen')
        report['inputs'][str(obj)] = hashlib.sha256(obj.read_bytes()).hexdigest()
        for reuse in [False, True]:
            for sanitize in [False, True]:
                label = f'{name}-{reuse}-{sanitize}'
                binary = out / label
                command = [str(obj) if item == old_object else item
                           for item in recipes['host-compile-link']
                           if not item.startswith('-DNEOCLR_HTTP_WORKERS=')]
                command[command.index('-o') + 1] = str(binary)
                command += ['-DNEOCLR_GC_OWNED_HEAP', '-DNEOCLR_HTTP_INVOCATIONS=2']
                if not reuse:
                    command += ['-DNEOCLR_ASYNC_CONTEXT_REUSE=0']
                if sanitize:
                    command += ['-fsanitize=undefined,bounds', '-fno-sanitize-recover=all']
                run(command, label + '-link')
                fault = name == 'RuntimeAwaitFault'
                result = run([str(binary)], label, expected=1 if fault else 0)
                if fault:
                    assert result.stderr.count('runtime async sample terminal fault') == 2, result.stderr
                    assert 'UNREACHABLE' not in result.stdout
                    assert 'Native HTTP host failed' not in result.stderr
                    assert 'runtime error:' not in result.stderr
                else:
                    assert result.stdout.count('PASS ') == 2, result.stdout
                    assert not result.stderr, result.stderr
                report['cases'].append(dict(fixture=name, reuse=reuse,
                                            sanitized=sanitize, invocations=2, passed=True))
    inputs = [Path(__file__).resolve(), args.build.resolve(),
              root / 'benchmarks/native-web/http-host.c',
              *(root / 'runtime/native').rglob('*.[ch]')]
    for command in report['commands']:
        for item in command['command']:
            path = Path(item)
            if path.is_absolute() and path.is_file():
                inputs.append(path)
    for path in inputs:
        report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed'] = True
finally:
    (out / 'qualification.json').write_text(json.dumps(report, indent=2) + '\n')
print('20 baseline/reuse optimized/sanitized runs, 40 invocations passed')
