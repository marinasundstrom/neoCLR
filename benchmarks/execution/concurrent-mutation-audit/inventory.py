#!/usr/bin/env python3
"""Reproducible search inventory, not proof of complete mutation-barrier coverage."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[3]
RULES = {
    'generated_store': r'\.store\s*\(',
    'generated_write_helper': r'\bwrite(?:_typed)?\s*\(',
    'native_bulk_memory': r'\b(?:memcpy|memmove|memset)\s*\(',
    'root_lifecycle': r'\bneoclr_(?:gc_host_root|roots_park|roots_resume|roots_unpark)[A-Za-z_0-9]*\s*\(',
}


def inventory():
    paths = subprocess.check_output(
        ['git', 'ls-files', 'runtime/codegen/src', 'runtime/native',
         'docs/experiments/jit-managed/host.c', 'benchmarks/native-web/http-host.c'],
        cwd=ROOT, text=True).splitlines()
    files = []
    for name in paths:
        path = ROOT / name
        if path.suffix not in {'.rs', '.c', '.h'}:
            continue
        data = path.read_bytes()
        hits = []
        for number, line in enumerate(data.decode().splitlines(), 1):
            for rule, pattern in RULES.items():
                if (rule.startswith('generated_') and path.suffix != '.rs') or (
                    rule == 'native_bulk_memory' and path.suffix == '.rs'):
                    continue
                if re.search(pattern, line):
                    hits.append(dict(line=number, rule=rule, text=line.strip()))
        files.append(dict(path=name, sha256=hashlib.sha256(data).hexdigest(), hits=hits))
    return dict(scope='Lexical candidates including reads, tests and declarations; ordinary C assignments, aliases and external providers require manual review.',
                rules=RULES, files=files,
                counts={rule: sum(hit['rule'] == rule for f in files for hit in f['hits']) for rule in RULES})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = inventory()
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report['counts']))
