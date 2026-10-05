"""Compile cumulative and artifact-referenced System source groups after the JSON gate.

This is a compilation frontier audit, not executable or production ownership evidence.
"""
import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'ownership', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--reference', type=Path, action='append', required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    original = json.loads(args.ownership.read_text())
    files = sorted((ROOT / 'runtime/raven/src/System').rglob('*.rvn'))
    all_sources = [str(p.relative_to(ROOT)) for p in files]
    baseline = sorted({p for lib in original['libraries'] for p in lib['sources']})
    operators = 'runtime/raven/src/System/ResultOperators.rvn'
    known = set(baseline) | {operators}
    adapters = [s for s in baseline if '/native/' in s]
    cases = []
    cumulative = []
    for library in original['libraries']:
        cumulative += library['sources']
        cases.append(('through-' + library['assemblyName'], sorted(set(cumulative)), False))
    cases.append(('cumulative-json', sorted(known), False))
    cases.append(('full-source', sorted(set(all_sources + adapters)), False))
    for label, folders in [('tasks', ('Tasks', 'Concurrency')),
                           ('storage', ('IO', 'Storage')),
                           ('http', ('Networking', 'Web', 'Tasks', 'Concurrency'))]:
        extra = {s for s in all_sources if any('/System/' + folder + '/' in s for folder in folders)}
        if label == 'http':
            extra.update('runtime/raven/src/System/' + n + '.rvn' for n in ('Uri', 'UriError'))
        cases.append(('cumulative-' + label, sorted(known | extra), False))
        cases.append(('imported-' + label, sorted(extra - known), True))

    probe = str((Path(__file__).parent / 'bootstrap/source-string-member-probe.rvn').relative_to(ROOT))
    cases.append(('source-string-member', sorted(set(original['libraries'][0]['sources'] + [probe])), False))
    cases.append(('imported-string-member', [probe], True))

    def run(case):
        name, sources, imported = case
        directory = output / name
        directory.mkdir()
        manifest = json.loads(json.dumps(original))
        if not imported:
            types = sorted({t for lib in original['libraries']
                            if set(lib['sources']).issubset(sources) for t in lib['types']})
            manifest['libraries'] = [dict(assemblyName='Numbers', sources=sources, types=types)]
            manifest['nativePrimitives'] = {k: 'Numbers' for k in manifest['nativePrimitives']}
            if 'runtime/raven/src/System/Boolean.rvn' not in sources:
                manifest['nativePrimitives'].pop('System.Boolean', None)
            manifest['typeOf'] = (dict(assemblyName='Numbers',
                typeInfoTypeName='System.Introspection.TypeInfo',
                contextTypeName='System.Runtime.RuntimeContext')
                if 'runtime/raven/src/System/Runtime/RuntimeContext.rvn' in sources else None)
        manifest_path = directory / 'ownership.json'
        manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
        artifact = directory / ('Additional.dll' if imported else 'Numbers.dll')
        command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--core-reference', str(args.core.resolve()),
                   '--runtime-seed', str(args.seed.resolve()), '--bootstrap-ownership', str(manifest_path),
                   '--bootstrap-intrinsics', '--library', '-o', str(artifact)]
        if imported:
            for reference in args.reference:
                command += ['--reference', str(reference.resolve())]
        command += sources
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as error:
            code, stdout, stderr = None, str(error.stdout or ''), str(error.stderr or '')
        diagnostics = re.findall(r'^error ([A-Z]+[0-9]+): (.*)$', stdout + '\n' + stderr, re.M)
        report = dict(name=name, sources=sources, sourceCount=len(sources), command=command,
            ownership=manifest, exitCode=code, outputPublished=artifact.exists(), stdout=stdout, stderr=stderr,
            errorCounts=dict(Counter(code for code, _ in diagnostics)),
            uniqueErrors=list(dict.fromkeys(message for _, message in diagnostics)))
        if artifact.exists():
            report['artifactSha256'] = hashlib.sha256(artifact.read_bytes()).hexdigest()
        (directory / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        print(name, code, report['errorCounts'], report['uniqueErrors'][:2], flush=True)
        return report

    with ThreadPoolExecutor(max_workers=3) as pool:
        reports = list(pool.map(run, cases))
    revision = lambda path: subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=path, text=True).strip()
    inputs = files + [ROOT / probe] + [ROOT / s for s in adapters] + args.reference + [args.compiler, args.core,
        args.seed, args.ownership, Path(__file__)] + [args.compiler.parent / name for name in
        ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    data = dict(runtimeRevision=revision(ROOT), compilerRevision=revision(args.compiler.resolve().parent),
        methodology='Compilation only; no source rewriting or execution. Cumulative cases merge accepted source groups into a diagnostic Numbers owner. Full-source includes bootstrap-owned declarations. Imported cases use the JSON gate artifacts without those sources. Binding failures mask later emission/runtime gaps; diagnostic counts are not independent defects.',
        sourceCount=len(files), hashes={str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},
        cases=reports)
    (output / 'audit.json').write_text(json.dumps(data, indent=2) + '\n')


if __name__ == '__main__':
    main()
