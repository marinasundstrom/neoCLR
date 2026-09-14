"""Inventory runtime source declarations for the Raven API coverage audit.

This is a source checklist, not a metadata reader or a claim of callable coverage.
Keep visibility candidates (including runtime service helpers) for explicit review.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, default=Path(__file__).with_name('runtime-api-inventory.json'))
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
entries = []
files = []
containers = {'.type', '.interface', '.delegate'}
blocks = containers | {'.method', '.property', '.function'}
leaves = {'.field', '.literal', '.enum'}

def visit(path, active=(), context=()):
    path = path.resolve()
    if path in active:
        raise ValueError('Include cycle: ' + str(path))
    files.append(str(path.relative_to(ROOT)))
    stack = list(context)
    initial_depth = len(stack)
    for line, text in enumerate(path.read_text().splitlines(), 1):
        text = text.split(';', 1)[0].strip()
        if not text:
            continue
        if text.startswith('.include '):
            match = re.fullmatch(r'\.include "([^"]+)"', text)
            if not match:
                raise ValueError((path, line, 'Unsupported include syntax'))
            visit(path.parent / match[1], (*active, path), tuple(stack))
            continue
        keyword = text.split()[0]
        if keyword == '.end':
            if not stack:
                raise ValueError((path, line, 'Unmatched .end'))
            stack.pop()
        elif keyword in blocks | leaves:
            hidden = any(item[2] for item in stack) or 'private' in text.split() or 'internal' in text.split()
            if not hidden:
                entries.append({
                    'file': str(path.relative_to(ROOT)), 'line': line,
                    'context': [item[1] for item in stack if item[0] in containers],
                    'declaration': text,
                    'role': 'runtime-service-review' if '/neoCLR/' in str(path) else 'library-api-review',
                })
            if keyword in blocks:
                stack.append((keyword, text, hidden))
    if len(stack) != initial_depth:
        raise ValueError((path, 'Unclosed declaration block'))

visit(ROOT / 'runtime/System.neoil')
result = {
    'purpose': 'Source declaration checklist; visibility and per-API Raven coverage need review. No entry asserts successful import or execution.',
    'manifest': 'runtime/System.neoil',
    'sourceFiles': files,
    'counts': dict(sorted(Counter(e['declaration'].split()[0][1:] for e in entries).items())),
    'declarations': entries,
}
# One declaration per line keeps generated diffs focused on changed signatures.
header = {key: value for key, value in result.items() if key != 'declarations'}
text = (json.dumps(header, indent=2)[:-2] + ',\n  "declarations": [\n'
        + ',\n'.join('    ' + json.dumps(entry) for entry in entries) + '\n  ]\n}\n')
if args.check:
    if args.output.read_text() != text:
        raise SystemExit('Runtime API inventory is stale; regenerate and review coverage changes.')
else:
    args.output.write_text(text)
print(f'{len(entries)} declaration candidates in {len(files)} source files; coverage review required')
