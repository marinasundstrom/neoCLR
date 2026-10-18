#!/usr/bin/env python3
"""Check phase sampling/accounting and measure empty-hook cost, not GC speed."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
from capture import instrument, ROOT, HERE

source = (ROOT / 'runtime/native/gc/native-gc.c').read_text()
assert instrument(source).count('phase_next(') == 5
try:
    instrument(source.replace('    /* Sweep and coalesce in one forward walk.', '    /* changed'))
except AssertionError:
    pass
else:
    raise AssertionError('changed instrumentation anchors must fail')

with tempfile.TemporaryDirectory() as directory:
    out = Path(directory)
    (out / 'test.c').write_text('''#include "phases.h"
#include <assert.h>
int main(void) {
    for (unsigned n = 0; n < 64000; n++) {
        phase_begin();
        for (unsigned i = 1; i < 6; i++) phase_next(i);
        phase_end(n == 63 ? 3 : 0);
    }
    assert(phase_calls == 64000 && phase_samples == 1000 && phase_errors == 1);
}
''')
    subprocess.run(['clang', '-O2', '-Wall', '-Wextra', '-Werror', '-I', str(HERE),
                    str(out / 'test.c'), '-o', str(out / 'test')], check=True)
    subprocess.run([str(out / 'test')], env=dict(os.environ, NEOCLR_GC_PHASES=str(out / 'report.json')), check=True)
    report = json.loads((out / 'report.json').read_text())
    assert report['samples'] == 1000 and report['errors'] == 1
    assert len(report['phases']) == 6
    print(json.dumps(report, indent=2))
