#!/usr/bin/env python3
"""Audit the post-Preview-12 System frontier; compilation results are not execution evidence."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'core', 'seed', 'libraries', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--runtime', type=Path, help='Assembler used by the explicit full-owned-handle case')
    parser.add_argument('--compiler-revision', required=True, help='Declared compiler source revision; hashes identify actual inputs')
    parser.add_argument('--case', action='append', help='Run selected cases; omitted controls are not revalidated')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    original = json.loads((ROOT / 'runtime/raven/native/poc-ownership.json').read_text())
    baseline = original['libraries'][0]['sources']
    production = sorted(str(p.relative_to(ROOT)) for p in (ROOT / 'runtime/raven/src/System').rglob('*.rvn'))
    network = [s for s in production if '/Networking/' in s or '/Web/' in s or s.endswith(('/Uri.rvn', '/UriError.rvn'))]
    network += ['runtime/raven/native/' + n + '.rvn' for n in ('RuntimeNetworkCalls', 'RuntimeNetworkServices')]
    selected = set(baseline + network)
    omitted = sorted(set(production) - selected)
    groups = {
        'storage': [s for s in omitted if '/Storage/' in s],
        'calendar': [s for s in omitted if Path(s).stem in ('DateTime', 'LocalTimeMapping', 'TimeZone', 'TimeZoneError', 'ZonedDateTime')],
        'services': [s for s in omitted if '/Console/' in s or '/Environment/' in s or '/Math/' in s or '/NativeMemory/' in s or s.endswith(('/GC.rvn', '/ConsoleReadError.rvn', '/EnvironmentError.rvn', '/System/Functions.rvn'))],
        'core': [s for s in omitted if Path(s).stem in ('Object', 'Value', 'Void', 'RuntimeTypeHandle', 'IntPtr', 'UIntPtr', 'UnionAttribute')],
    }
    groups['contracts'] = sorted(set(omitted) - set(sum(groups.values(), [])))
    cases = [('baseline', baseline, False), ('network', network, True)]
    cases += [(name, sources, True) for name, sources in groups.items()]
    adapters = sorted(str(p.relative_to(ROOT)) for p in (ROOT / 'runtime/raven/native').glob('*.rvn'))
    storage_adapters = ['runtime/raven/native/' + n + '.rvn' for n in ('RuntimeStorageCalls', 'RuntimeStorageServices', 'RuntimeFileTextServices')]
    cases += [('storage-with-adapters', groups['storage'] + storage_adapters, True)]
    cases += [('full-source', production + adapters, False),
              ('full-bootstrap-handle', [s for s in production + adapters if not s.endswith('/RuntimeTypeHandle.rvn')], False)]
    if args.runtime or 'full-owned-handle' in (args.case or []):
        cases.append(('full-owned-handle', production + adapters, False))
    if args.case:
        unknown = set(args.case) - {name for name, _, _ in cases}
        if unknown:
            parser.error('Unknown cases: ' + ', '.join(sorted(unknown)))
        cases = [case for case in cases if case[0] in args.case]
    if any(name == 'full-owned-handle' for name, _, _ in cases) and not args.runtime:
        parser.error('full-owned-handle requires --runtime to assemble its retained seed')
    inputs = [args.compiler, args.core, args.seed, args.libraries / 'Numbers.dll', args.libraries / 'Http.dll', Path(__file__), ROOT / 'runtime/raven/native/poc-ownership.json']
    inputs += [args.compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]
    inputs += [ROOT / s for s in production + adapters]
    if args.runtime:
        inputs += [args.runtime, ROOT / 'runtime/raven/native/poc-seed.neoil']
    report = dict(sourceRevision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        controlsIncluded=[name for name, _, _ in cases if name in ('baseline', 'network')], declaredCompilerRevision=args.compiler_revision, productionFiles=len(production), selectedProductionFiles=len(set(production) & selected),
        omitted=omitted, groups=groups, scope='Compilation frontier only. Bootstrap inputs retained; full-source owner is diagnostic, not a proposed production layout.',
        hashes={str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, cases=[])
    for name, sources, imported in cases:
        directory = output / name
        directory.mkdir()
        manifest = json.loads(json.dumps(original))
        if not imported:
            manifest['libraries'][0]['sources'] = sources
        seed = args.seed.resolve()
        if name == 'full-owned-handle':
            library = manifest['libraries'][0]
            manifest['failure'] = dict(assemblyName=library['assemblyName'], namespaceName='System', functionName='Fail')
            library['types'] = sorted(set(library['types']) | {'System.RuntimeTypeHandle'})
            manifest['nativePrimitives']['System.RuntimeTypeHandle'] = library['assemblyName']
            # Reuse the already implemented source-handle ownership contract. The
            # exact empty seed declaration must disappear when this owner is selected.
            seed_text = (ROOT / 'runtime/raven/native/poc-seed.neoil').read_text()
            declaration = '.type System.RuntimeTypeHandle\n.sealed\n.end\n'
            if seed_text.count(declaration) != 1:
                raise ValueError('Expected exactly one retained handle declaration')
            seed_source = directory / 'System.neoil'
            seed_source.write_text(seed_text.replace(declaration, ''))
            seed = directory / 'System.neox'
            assembled = subprocess.run([str(args.runtime.resolve()), 'assemble', str(seed_source), str(seed), '--format', 'neox'],
                                       cwd=ROOT, capture_output=True, text=True, timeout=60)
            if assembled.returncode:
                raise RuntimeError(assembled.stdout + assembled.stderr)
            report['handleSeedAssembly'] = dict(command=assembled.args, exitCode=assembled.returncode,
                                               stdout=assembled.stdout, stderr=assembled.stderr)
            for path in (seed_source, seed):
                report['hashes'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        ownership = directory / 'ownership.json'
        ownership.write_text(json.dumps(manifest, indent=2) + '\n')
        report['hashes'][str(ownership)] = hashlib.sha256(ownership.read_bytes()).hexdigest()
        artifact = directory / ('Additional.dll' if imported else 'Numbers.dll')
        command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--core-reference', str(args.core.resolve()), '--runtime-seed', str(seed),
            '--bootstrap-ownership', str(ownership), '--bootstrap-intrinsics', '--library', '-o', str(artifact)]
        if imported:
            command += ['--reference', str((args.libraries / 'Numbers.dll').resolve())]
            if name != 'network':
                command += ['--reference', str((args.libraries / 'Http.dll').resolve())]
        if name.startswith('full-'):
            command += ['--source-object-root']
        command += sources
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=300)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as error:
            code = None
            stdout = error.stdout.decode(errors='replace') if isinstance(error.stdout, bytes) else error.stdout or ''
            stderr = error.stderr.decode(errors='replace') if isinstance(error.stderr, bytes) else error.stderr or ''
        diagnostics = re.findall(r'^error ([A-Z]+[0-9]+): (.*)$', stdout + '\n' + stderr, re.M)
        outcome = 'emitted' if code == 0 and artifact.exists() else 'timeout' if code is None else 'compiler-crash' if code < 0 or 'Unhandled exception.' in stderr else 'rejected'
        entry = dict(name=name, outcome=outcome, sourceCount=len(sources), sources=sources, command=command, exitCode=code, outputPublished=artifact.exists(), stdout=stdout, stderr=stderr,
            errorCounts=dict(Counter(c for c, _ in diagnostics)), uniqueErrors=list(dict.fromkeys(m for _, m in diagnostics)))
        if artifact.exists():
            entry['artifactSha256'] = hashlib.sha256(artifact.read_bytes()).hexdigest()
        report['cases'].append(entry)
        (output / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
        print(name, code, entry['errorCounts'], flush=True)
    # Expected failures are the purpose of the inventory; baseline failures invalidate it.
    if any(c['exitCode'] != 0 or not c['outputPublished'] for c in report['cases'] if c['name'] in ('baseline', 'network')):
        raise RuntimeError('Released compilation controls failed; do not interpret frontier results')


if __name__ == '__main__':
    main()
