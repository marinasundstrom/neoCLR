"""Pair eager/lazy JIT against identical, hash-verified retained HTTP hosts and inputs.

The retained native binaries are controls, not a rebuild of current native sources.
Compiler sources and the new runner are pinned separately in this manifest.
"""
import argparse
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--placement', choices=['default','system','fallback'], default='default')
a = p.parse_args()
root = Path(__file__).resolve().parents[3]
source = a.build.resolve() / 'build.json'
built = json.loads(source.read_text())
assert built['passed']
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
r = dict(passed=False, scope=__doc__, codePlacement=a.placement, retainedHostManifest=str(source), recipes={}, inputs={})
def pin(path):
    path = Path(path).resolve()
    r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()

for collector in ['sync', 'live']:
    old = built['recipes'][collector + '-jit']
    selection_path = Path(old['env']['NEOCLR_LIVE_SELECTION_COMMAND'])
    assert hashlib.sha256(selection_path.read_bytes()).hexdigest() == built['inputs'][str(selection_path)]
    selection = json.loads(selection_path.read_text())
    # Only consumed files, not historical build-time source files. Every retained
    # guest, module, policy and host must still match the original manifest.
    for value in selection[1:]:
        path = Path(value)
        if path.is_absolute() and path.is_file():
            expected = built['inputs'].get(str(path))
            if expected is None:
                manifest = path.parent/'bundle.json'
                assert manifest.is_file(), ('untracked retained input', path)
                expected = json.loads(manifest.read_text())['files'][path.name]
                pin(manifest)
            assert expected is not None, ('untracked retained input', path)
            assert hashlib.sha256(path.read_bytes()).hexdigest() == expected, ('stale artifact', path)
            pin(path)
    pin(selection_path)
    for lazy in [False, True]:
        command = old['command'].copy()
        command[0] = str(a.runner.resolve())
        env = dict(old['env'], NEOCLR_JIT_LAZY=str(int(lazy)))
        env.pop('NEOCLR_TEST_JIT_ARENA', None)
        if a.placement != 'default': env['NEOCLR_TEST_JIT_ARENA'] = a.placement
        r['recipes'][('lazy-' if lazy else 'eager-') + collector + '-jit'] = dict(old, command=command, env=env)
for path in [source, a.runner, Path(__file__), root/'Cargo.lock', *[root/'runtime/codegen/src'/n for n in ['jit.rs','jit_lazy.rs','values.rs','fault_details.rs','jit_memory.rs','jit_memory_fallback.rs','lib.rs','mutation_admission.rs','mutation_store_tests.rs','gc_probe.rs','gc_points.rs']]]:
    pin(path)
r['passed'] = True
(out/'build.json').write_text(json.dumps(r, indent=2)+'\n')
