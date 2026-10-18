#!/usr/bin/env python3
"""Compile copied consumer fixtures with a selected bundle; compare its backend with a candidate."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--backend', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--interpreter', type=Path, help='Candidate runtime for interpreter comparisons; defaults to installed runtime')
p.add_argument('--cases', nargs='+', default=['wrapper', 'value', 'pattern', 'record-construction', 'record-json', 'record-display', 'success'])
a = p.parse_args()
bundle = a.bundle.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
copy = out/'bundle'
copy.mkdir()
for path in bundle.iterdir():
    if path.name != 'samples':
        (copy/path.name).symlink_to(path, target_is_directory=path.is_dir())
folder = copy/'samples/tasks'
folder.mkdir(parents=True)
shutil.copyfile(bundle/'samples/tasks/App.rvnproj', folder/'App.rvnproj')
spec = importlib.util.spec_from_file_location('sample', bundle/'tools/run-native-sample.py')
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
runtime = bundle/'bin/neoclr'
interpreter = a.interpreter.resolve() if a.interpreter else runtime
report = dict(passed=False, bundle=str(bundle), backend=str(a.backend.resolve()), commands=[], inputs={})
for path in [bundle/'bin/neoclr-jit-host.dylib', runtime, interpreter, bundle/'bin/neoclr-codegen', bundle/'sdk/tools/rvnc/rvnc.dll', a.backend.resolve(), bundle/'manifest.json']:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

def run(label, command):
    command = list(map(str, command))
    result = subprocess.run(command, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(label=label, command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    return result

host = bundle/'bin/neoclr-jit-host.dylib'
launcher = out/'aot-main.c'
launcher.write_text("""#include <stdint.h>
extern int32_t neoclr_entry_v4(int32_t, int32_t *, void *);
extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);
int main(int argc, const char **argv) {
    return neoclr_jit_run_v1((const void *)&neoclr_entry_v4, 0, argc, argv);
}
""")
flags = '--compile-system --bind-user-fault --bind-stack-trace --reference-arena --native-gc --bind-console-read-byte --bind-console-write-line --bind-console-stream-output --bind-int32-to-string --bind-utf8-text --native-stack-budget --bind-integer-text --bind-character-text --bind-system-culture --bind-time-zone-rules --bind-cryptography --bind-tls --bind-task-queue --bind-scheduler --bind-timers --bind-socket-listener --bind-socket-accept --bind-socket-transfer --bind-socket-client --bind-runtime-async'.split()
try:
    for name in a.cases:
        expression = {'wrapper': 'Console.WriteLine(result)', 'value': 'Console.WriteLine(result.Value)',
                      'pattern': 'if let .Ok(text) = result {\n        Console.WriteLine(text)\n    }'}.get(name, '')
        source = '''import System.*
import System.Data.Json.*
import System.Concurrency.Tasks.*
class Person {
    var Name: string = ""
    var Age: int = 0
}
async func Main() -> Task<unit> {
    let result = JsonSerializer.Serialize(Person { Name = "Bob", Age = 42 })
    ''' + expression + '\n}\n'
        if name == 'success':
            source = source.replace('JsonSerializer.Serialize(Person { Name = "Bob", Age = 42 })', 'JsonSerializer.Serialize("Bob")').replace('    \n}', '    Console.WriteLine(result)\n}')
        if name.startswith('record-'):
            source = """import System.*
import System.Data.Json.*
record struct Person(val Name: string, val Age: int)
func Main() {
"""
            if name == 'record-display':
                source += '    Console.WriteLine(Person("Bob", 42))\n'
            elif name == 'record-construction':
                source += '    let person = Person("Bob", 42)\n    Console.WriteLine(person.Name)\n'
            else:
                source += '    let result = JsonSerializer.Serialize(Person("Bob", 42))\n    if let .Error(error) = result {\n        Console.WriteLine("unsupported record mapping")\n    }\n'
            source += '}\n'
        (folder/'library-async.rvn').write_text(source)
        (out/(name+'.rvn')).write_text(source)
        built = run(name+'-compile', ['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',folder/'App.rvnproj'])
        assert built.returncode == 0
        assembly = folder/'bin/neoclr/App.dll'
        roots = [] if name.startswith('record-') else helper.reflection_roots(runtime, assembly, ['Person'], lambda c: run(name+'-metadata',c).check_returncode())
        command = [runtime,'run',assembly,*roots]
        old = run(name+'-preview16', command)
        if name in ['wrapper', 'value']:
            assert old.returncode != 0 and 'Object display with boxing or arrays' in old.stderr
        if name == 'record-json':
            assert old.returncode != 0 and 'record boxing currently requires empty value records' in old.stderr
        if name == 'record-construction':
            assert old.returncode == 0 and old.stdout == 'Bob\n'
        new = run(name+'-candidate', [*command,'--jit-compiler',a.backend.resolve()])
        interpreted = run(name+'-interpreter', [interpreter,'run',assembly,'--execution','interpreter','--system',bundle/'lib/System.runtime.neox','--object-root',bundle/'lib/System.Runtime.dll',*[part for module in json.loads((bundle/'manifest.json').read_text())['modules'] for part in ['--module',bundle/module]]])
        assert new.returncode == interpreted.returncode == 0, (new.stderr, interpreted.stderr)
        assert new.stdout == interpreted.stdout, (new.stdout, interpreted.stdout)
        obj, executable = out/(name+'.o'), out/(name+'-aot')
        modules = [part for module in json.loads((bundle/'manifest.json').read_text())['modules'] for part in ['--module',bundle/module]]
        compiled = run(name+'-aot-compile', [a.backend.resolve(), '--closed-world', assembly, '@entry', obj,
            '--system',bundle/'lib/System.runtime.neox','--object-root',bundle/'lib/System.Runtime.dll',*modules,*flags,*roots])
        assert compiled.returncode == 0, compiled.stderr
        linked = run(name+'-aot-link', ['clang',launcher,obj,host,'-o',executable])
        assert linked.returncode == 0, linked.stderr
        # The published dylib has a build-time install name. Change only the copied
        # executable's dependency to the exact installed host, never the bundle.
        identity = run(name+'-host-id', ['otool','-D',host]).stdout.splitlines()[1]
        run(name+'-host-bind', ['install_name_tool','-change',identity,host,executable]).check_returncode()
        aot = run(name+'-aot', [executable])
        assert (aot.returncode, aot.stdout) == (interpreted.returncode, interpreted.stdout), (aot.stderr, aot.stdout)
        if name == 'record-json':
            assert new.stdout == 'unsupported record mapping\n'
    report['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(report, indent=2)+'\n')
