"""Check that the selected Raven profile contains no handwritten managed bodies.

This complements metadata admission and snapshot hashes: it compares normalized
method/function text with generated fragments and the explicit native-service files.
The historical Neo profile is intentionally outside this source-ownership check.
"""
import re
from collection_library import ROOT, build


def bodies(text):
    current = []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith(';'):
            continue
        if re.match(r'^\.(method|function) ', line):
            if current:
                raise ValueError('Nested method/function declaration')
            current = [line]
        elif current:
            current.append(line)
            if line == '.end':
                yield '\n'.join(current)
                current = []
    if current:
        raise ValueError('Unclosed method/function declaration')


def catalog(paths):
    return {body for path in paths for body in bodies(path.read_text())}


generated = catalog((ROOT / 'runtime/raven/generated').glob('*.neoil'))
native = catalog((ROOT / 'runtime/neoCLR').rglob('*.neoil'))
selected = list(bodies(build(ROOT / 'runtime/System.neoil')))
unknown = [body.splitlines()[0] for body in selected if body not in generated | native]
if unknown:
    raise SystemExit('Unowned Raven-profile bodies:\n' + '\n'.join(unknown))
print(f'All {len(selected)} Raven-profile method/function declarations come from '
      f'generated snapshots or explicit runtime services '
      f'({sum(body in native for body in selected)} service declarations)')
