"""Use production codegen optimization defaults in a pinned development harness.

This removes only the test optimization override. Explicit arena controls and
private concurrent-GC host qualification remain visible in each recipe.
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
assert build['passed']
for path, digest in build['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
for recipe in build['recipes'].values():
    recipe['env'].pop('NEOCLR_TEST_OPT_LEVEL', None)
build.update(scope=__doc__, backendOptimization='cranelift-default', qualificationRunner=True)
for path in [source, Path(__file__).resolve()]:
    build['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
(out/'build.json').write_text(json.dumps(build, indent=2)+'\n')
