"""Expand a verified lazy-JIT build into paired binding/placement diagnostic controls.

All variants share one compiler and retained guest/native hosts. The control keeps
all direct calls through trampolines. Explicit placement controls override the host default. Direct binding is test-only.
"""
import argparse
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
source = a.build.resolve()/'build.json'
build = json.loads(source.read_text())
assert build['passed'] and build['codePlacement'] == 'system'
for path, digest in build['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
recipes = dict(build['recipes'])
for collector in ['sync', 'live']:
    eager = recipes['eager-'+collector+'-jit']
    recipes['arena-eager-'+collector+'-jit'] = dict(eager, env=dict(eager['env'], NEOCLR_TEST_JIT_ARENA='fallback'))
    base = recipes['lazy-'+collector+'-jit']
    recipes['lazy-'+collector+'-jit'] = dict(base, env=dict(base['env'], NEOCLR_TEST_LAZY_BIND_COMPILED='1'))
    for variant, extra in [('control', {}),
                           ('arena', {'NEOCLR_TEST_JIT_ARENA': 'fallback', 'NEOCLR_TEST_LAZY_BIND_COMPILED': '1'}),
                           ('arena-control', {'NEOCLR_TEST_JIT_ARENA': 'fallback'})]:
        recipes[variant+'-'+collector+'-jit'] = dict(base, env=dict(base['env'], **extra))
build.update(scope=__doc__, recipes=recipes)
for path in [source, Path(__file__).resolve()]:
    build['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(build, indent=2)+'\n')
