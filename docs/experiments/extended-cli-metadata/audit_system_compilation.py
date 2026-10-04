"""Audit native System compilation frontiers; binding failures mask later phases.

Uses unchanged sources and the explicit accepted bootstrap. Family selections are
diagnostic source sets, not proposed production assembly ownership.
"""
import argparse
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    files = sorted((ROOT / 'runtime/raven/src/System').rglob('*.rvn'))
    relative = lambda p: str(p.relative_to(ROOT))
    source = {relative(p): p.read_text() for p in files}
    manifest = HERE / 'bootstrap/offset-ownership.json'
    baseline = json.loads(manifest.read_text())['libraries'][0]['sources']
    prefix = 'runtime/raven/src/System/'
    names = lambda *xs: [prefix + x + '.rvn' for x in xs]
    folder = lambda name: [p for p in source if p.startswith(prefix + name + '/')]
    primitives = names('Object', 'Value', 'Void', 'String', 'Char', 'Boolean', 'Byte',
                       'SByte', 'Int16', 'UInt16', 'Int32', 'UInt32', 'Int64', 'UInt64',
                       'Single', 'Double', 'IntPtr', 'UIntPtr', 'RuntimeTypeHandle',
                       'Runtime/CompilerServices/UnionAttribute')
    cases = {
        'full-source': list(source),
        'nonprimitive-source': [p for p in source if p not in primitives],
        'memory-stream': names('IO/InputStream', 'IO/OutputStream', 'IO/SeekableStream', 'IO/StreamError', 'IO/MemoryStream'),
        'numeric-foundation': primitives + names('Number', 'NumberParseError', 'BooleanParseError', 'IntegerDivisionError', 'Text/Utf8SliceError', 'StringComparison'),
        'text': folder('Text') + names('IO/InputStream', 'IO/OutputStream', 'IO/StreamError'),
        'storage-io': folder('IO') + folder('Storage') + folder('Text'),
        'tasks-workers': folder('Tasks') + folder('Concurrency'),
        'network-http': folder('Networking') + folder('Web') + folder('Tasks') + folder('Concurrency') + names('Uri', 'UriError'),
        'introspection': folder('Introspection') + folder('Runtime/Reflection') + names('Runtime/RuntimeContext'),
        'json': folder('Data') + folder('Introspection') + folder('Runtime/Reflection') + names('Runtime/RuntimeContext'),
        'namespace-functions': names('Functions', 'Console/Functions', 'Console/Streams', 'Environment/Functions', 'EnvironmentError', 'ConsoleReadError'),
        'operators': names('OptionOperators', 'ResultOperators'),
    }
    probes = {
        'probe-unit-value': 'import System.*\nfunc Read() -> Result<unit, string> => .Ok(())\n',
        'probe-float': 'func Read(value: double) -> double => value + 1.0\n',
        'probe-enum': 'public enum Marker { One, Two }\n',
        'probe-inheritance': 'public open class Base { public init() {} }\npublic class Derived : Base { public init() {} }\n',
        'probe-self': 'public interface Copyable { func Copy() -> Self }\n',
        'probe-mutable-capture': 'func Read() -> int {\n var count = 0\n let callback: () -> int = () => { count += 1; return count }\n return callback()\n}\n',
    }
    for name, text in probes.items():
        path = args.output / (name + '.rvn')
        path.write_text(text)
        cases[name] = [str(path.resolve())]
    common = ['dotnet' , str(args.compiler.resolve()), 'neoclr', '--core-reference', str(args.core.resolve()),
              '--runtime-seed', str(args.seed.resolve()), '--bootstrap-ownership', str(manifest),
              '--bootstrap-intrinsics', '--library']
    def run(item):
        name, additions = item
        selected = sorted(set(baseline + additions))
        directory = args.output / name
        directory.mkdir()
        artifact = directory / 'NeoCLR.Collections.dll'
        command = common + ['-o', str(artifact)] + selected
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as error:
            code, stdout, stderr = None, str(error.stdout or ''), str(error.stderr or '')
        diagnostics = re.findall(r'^error ([A-Z]+[0-9]+): (.*)$', stdout + '\n' + stderr, re.M)
        report = dict(name=name, sources=selected, command=command, exitCode=code,
                      outputPublished=artifact.exists(), stdout=stdout, stderr=stderr,
                      errorCounts=dict(Counter(code for code, _ in diagnostics)),
                      uniqueErrors=list(dict.fromkeys(message for _, message in diagnostics)))
        (directory / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        return report
    with ThreadPoolExecutor(max_workers=3) as pool:
        reports = list(pool.map(run, cases.items()))
    callers = defaultdict(set)
    features = defaultdict(list)
    patterns = {
        'erased-value-operations': r'RuntimeServices\.(?:UnpackValue|PackValue|IsValue)\b',
        'inheritance-and-overrides': r'\b(?:abstract|override|base)\b',
        'protected-members': r'\bprotected\b',
        'async-await': r'\b(?:async|await)\b',
        'self-signatures': r'\bSelf\b',
        'runtime-services': r'\bRuntimeServices\.',
        'unit-value-spelling': r'\bunit\b',
        'enum-declarations': r'\benum\s+\w+',
    }
    for path, text in source.items():
        text = re.sub(r'//[^\n]*|/\*.*?\*/', '', text, flags=re.S)
        for member in re.findall(r'\bRuntimeServices\.(\w+)', text):
            callers[member].add(path)
        for name, pattern in patterns.items():
            if re.search(pattern, text):
                features[name].append(path)
    revision = lambda path: subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
    data = dict(
        methodology='Frontier audit only. Unchanged source sets over the accepted 48-source baseline; current core/seed. Lexical caller counts are potential reach, not guaranteed unlock counts. Binding failures mask emission/runtime failures. No failed compilation is executed. Family sets are not final ownership manifests.',
        runtimeRevision=revision(ROOT), compilerRevision=revision(args.compiler.parent),
        sourceCount=len(files), baselineCount=len(baseline), probes=probes,
        inputs=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest())
                for p in files + [args.compiler, args.core, args.seed, manifest, Path(__file__)]],
        serviceCallers={name: sorted(paths) for name, paths in sorted(callers.items(), key=lambda pair: (-len(pair[1]), pair[0]))},
        featureFiles=dict(features), cases=reports)
    (args.output / 'audit.json').write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps(dict(sourceCount=len(files), baselineCount=len(baseline),
                          cases=[dict(name=r['name'], exitCode=r['exitCode'], errors=r['errorCounts'], first=r['uniqueErrors'][:3]) for r in reports],
                          serviceCount=len(callers), serviceFiles=len(set().union(*callers.values()))), indent=2))


if __name__ == '__main__':
    main()
