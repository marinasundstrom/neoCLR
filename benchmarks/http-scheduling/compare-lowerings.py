#!/usr/bin/env python3
"""Build and validate matched application async lowerings; libraries stay fixed."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('concurrency', ROOT/'benchmarks/http-concurrency/validate.py')
validation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validation)
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--recipe', type=Path, required=True)
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    recipe = json.loads(a.recipe.read_text())
    assert recipe['passed'] and recipe['profile'] == 'macos-arm64-http-v1'
    bundle, out = a.bundle.resolve(), a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    provider = ROOT/'runtime/raven/compiler-services/RuntimeAwait.rvn'
    carrier = ROOT/'docs/experiments/runtime-async/native-runtime.c'
    compiler = bundle/'sdk/tools/rvnc/rvnc.dll'
    report = dict(passed=False, commands=[], inputs={}, lowerings={}, runs=[],
        scope='Matched application lowering only; dependency libraries unchanged. macOS ARM64 AOT concurrent HTTP qualification; phase times are not isolated scheduler latency or throughput.')
    for path in [a.recipe, Path(__file__), Path(validation.__file__), ROOT/'benchmarks/http-concurrency/Server.rvn', provider, carrier, compiler]:
        report['inputs'][str(path.resolve())] = sha(path)
    def run(command, label, env=None):
        result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=600, env=env)
        log = out/(label+'.stdout')
        log.write_text(result.stdout)
        report['commands'].append(dict(label=label, command=[str(x) for x in command], exitCode=result.returncode,
                                       stdoutFile=str(log), stdoutSha256=sha(log), stderr=result.stderr))
        if result.returncode:
            raise RuntimeError(label+': '+result.stderr[-3000:]+result.stdout[-1000:])
        return result.stdout
    try:
        native_base = next(c['command'] for c in recipe['commands'] if c.get('phase') == 'native-compilation')
        link_base = next(c['command'] for c in recipe['commands'] if c.get('phase') == 'host-compile-link')
        old_bundle = str(Path(native_base[0]).parent.parent)
        for lowering in ['state-machine', 'runtime']:
            directory = out/lowering
            directory.mkdir()
            helper = '<RavenNeoClrRuntimeAsyncHelper>RuntimeAwaitProvider.Bridge</RavenNeoClrRuntimeAsyncHelper>' if lowering == 'runtime' else '<RavenNeoClrRuntimeAsyncHelper></RavenNeoClrRuntimeAsyncHelper>'
            project = directory/'Probe.rvnproj'
            project.write_text(f'''<Project Sdk="Microsoft.NET.Sdk">
<Import Project="{bundle}/lib/NeoCLR.ClassLibrary.props" />
<PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType>
<AssemblyName>HttpScheduling</AssemblyName>{helper}</PropertyGroup>
<ItemGroup><Compile Include="{ROOT}/benchmarks/http-concurrency/Server.rvn" />
<Compile Include="{provider}" Condition="'$(RavenNeoClrRuntimeAsyncProviderImported)' != 'true'" /></ItemGroup></Project>''')
            env = dict(os.environ, NeoClrBundleRoot=str(bundle))
            stdout = run(['dotnet', compiler, 'neoclr', '--project', project], lowering+'-source', env)
            paths = [line.removeprefix('Native build output: ') for line in stdout.splitlines() if line.startswith('Native build output: ')]
            assembly, obj = directory/'app.dll', directory/'app.o'
            shutil.copyfile(paths[-1], assembly)
            native = [v.replace(old_bundle, str(bundle)) for v in native_base]
            native[2], native[4] = str(assembly), str(obj)
            for flag in ['--bind-scheduler', '--bind-runtime-async']:
                if flag not in native: native.append(flag)
            selection = json.loads(run(native, lowering+'-native'))
            bindings = [b for b in selection['nativeBindings'] if b.get('implementation', '').startswith('runtime-async-')]
            assert bool(bindings) == (lowering == 'runtime'), bindings
            link = [str(obj) if v.endswith('/app.o') else v for v in link_base]
            link[link.index('-o')+1] = str(directory/'app')
            link += ['-DNEOCLR_RUNTIME_ASYNC', '-I', str(carrier.parent), str(carrier), '-Wno-deprecated-declarations']
            run(link, lowering+'-link')
            build = dict(passed=True, profile=recipe['profile'], commands=[dict(phase='host-compile-link', command=link)])
            (directory/'build.json').write_text(json.dumps(build, indent=2)+'\n')
            report['lowerings'][lowering] = dict(runtimeAsyncBindings=bindings, assemblySha256=sha(assembly), objectSha256=sha(obj), executableSha256=sha(directory/'app'))
            for value in [*native, *link, str(project)]:
                if Path(value).is_file(): report['inputs'][str(Path(value).resolve())] = sha(value)
        for index, hold in enumerate([0, .1]):
            for lowering in ['state-machine', 'runtime'][::1 if index == 0 else -1]:
                result = validation.run_case([str(out/lowering/'app')], slow_hold_seconds=hold)
                report['runs'].append(dict(lowering=lowering, holdSecondsPerPhase=hold, validation=result))
                print('PASS', lowering, hold, flush=True)
        for path, digest in report['inputs'].items(): assert sha(path) == digest, path
        report['passed'] = True
    finally:
        (out/'comparison.json').write_text(json.dumps(report, indent=2)+'\n')

if __name__ == '__main__': main()
