"""The selected policy must reject missing, conflicting and shared-heap knobs."""
from pathlib import Path
import runpy

root = Path(__file__).resolve().parents[3]
policy = runpy.run_path(str(root/'runtime/native/gc/concurrent/policy.py'))
validate = policy['validate_flags']
flags = ['-D'+value for value in policy['POLICY']['defines']]
validate(['clang', *flags, '-DNEOCLR_RUNTIME_ASYNC', '-O2'])
rejected = 0
for command in [flags[1:], flags+['-DNEOCLR_WORKER_GC'],
                flags+['-DNEOCLR_GATE_WORKERS'], flags+['-DNEOCLR_LIVE_BATCH'],
                flags+['-DNEOCLR_GC_ALLOCATION_BUDGET=64'], flags+[flags[0]],
                flags+['-UNEOCLR_GC_PACED'], flags+['-D', 'NEOCLR_WORKER_GC']]:
    try:
        validate(command)
    except ValueError:
        rejected += 1
    else:
        raise AssertionError(command)
assert rejected == 8
print('PASS selected policy and eight incompatible controls')
