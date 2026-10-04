#!/usr/bin/env python3
"""Execute snapshot-imported and independently authored grapheme references.

First run the C# metadata suite with NEOCLR_GRAPHEME_ARTIFACT=<artifact>.
Use a fresh output directory; runtime assembly publication refuses overwrites.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--artifact', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
bootstrap = Path(__file__).resolve().parent
results = []


def run(arguments, expected=0):
    command = [str(args.runtime.resolve()), *map(str, arguments)]
    process = subprocess.run(command, capture_output=True, text=True)
    results.append(dict(command=command, exit=process.returncode,
                        stdout=process.stdout, stderr=process.stderr))
    if process.returncode != expected or (expected == 42 and process.stdout):
        raise RuntimeError(results[-1])


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


system = args.output / 'System.neox'
run(['assemble', bootstrap / 'metadata-empty-system.neoil', system, '--format', 'neox'])
artifacts = [args.artifact, system, args.runtime]
for mode in ('authored', 'imported'):
    library = Path(str(args.artifact) + '.' + mode)
    manifest = json.loads(Path(str(library) + '.json').read_text())
    forward = next(f['name'] for f in manifest['functions'] if f['origin']['name'] == 'Forward')
    source = args.output / (mode + '.neoil')
    template = (bootstrap / 'metadata-grapheme-consumer.neoil').read_text()
    template = template.replace('ldloca value\ncall instance System.Char::Echo()',
                                f'ldloc value\ncall {forward}(Char)')
    source.write_text(template)
    output = args.output / (mode + '.neox')
    dependencies = ['--module', args.artifact, '--module', library, '--system', system]
    run(['assemble', source, output, '--format', 'neox', *dependencies])
    run(['verify', output, *dependencies])
    run(['run', output, *dependencies], 42)
    artifacts.extend([library, source, output])
evidence = dict(results=results, artifacts={str(p): digest(p) for p in artifacts})
(args.output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')
print('Both separately compiled grapheme consumers verified and exited 42.')
