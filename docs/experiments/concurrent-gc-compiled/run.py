#!/usr/bin/env python3
"""Qualify AOT-generated callback dispatch against the isolated concurrent GC POC."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
POC = HERE.parent / 'concurrent-gc-walker'
WORKER = HERE.parent / 'concurrent-gc-worker'
BASE = HERE.parent / 'aot-console'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--aot', type=Path, required=True)
    p.add_argument('--validated-build', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build, aot = a.validated_build.resolve(), a.aot.resolve()
    qualification = json.loads((build / 'results.json').read_text())
    assert qualification['passed']
    for name, digest in qualification['inputs'].items():
        assert sha(Path(name)) == digest, name
    inputs = [Path(__file__).resolve(), HERE / 'test.c', HERE / 'callbacks.neoil',
              BASE / 'native-session.h', BASE / 'text-arena.c',
              HERE.parent / 'aot-fault-details/render.c',
              HERE.parent / 'aot-fault-details/fault-details.h',
              *sorted((ROOT / 'runtime/native').rglob('*.[ch]'))]
    report = dict(passed=False, productionReady=False, platform=platform.platform(),
                  qualification=qualification, aotSha256=sha(aot), commands=[],
                  inputs={str(f): sha(f) for f in inputs})
    def run(command):
        r = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=60)
        report['commands'].append(dict(command=r.args, exitCode=r.returncode,
                                       stdout=r.stdout, stderr=r.stderr))
        r.check_returncode()
        return r.stdout
    try:
        seed = out / 'System.neoil'
        seed.write_text('.module System\n.references ()\n')
        helper = out / 'Helpers.neoil'
        helper.write_text('.module Helpers\n.references ()\n')
        obj = out / 'guest.o'
        run([aot, '--closed-world', HERE / 'callbacks.neoil', 'Calculate', obj,
             '--system', seed, '--module', helper, '--compile-system',
             '--reference-arena', '--native-gc', '--native-host-bootstrap'])
        roots = (build / 'root-probe.c').read_text()
        old = 'if (neoclr_gc_collect_v1((neoclr_aot_context *)frame->context, head)) abort();'
        assert roots.count(old) == 1
        roots = roots.replace(old, 'if (experiment_gc_boundary((neoclr_aot_context *)frame->context, head)) abort();')
        roots = '#include "native-gc.h"\nextern int32_t experiment_gc_boundary(neoclr_aot_context *, const neoclr_probe_frame *);\n' + roots
        (out / 'root-probe.c').write_text(roots)
        for defer in (0, 1):
            binary = out / ('deferred' if defer else 'eager')
            run(['clang', '-std=c11', '-O2', '-g', '-Wall', '-Wextra', '-Werror',
                 '-pthread', '-fsanitize=undefined,bounds', '-DNEOCLR_NATIVE_GC',
                 '-DEXPERIMENT_DEFER_BOUNDARIES=' + str(defer),
                 '-I', build, '-I', POC, '-I', WORKER, '-I', ROOT / 'runtime/native/gc',
                 '-I', BASE, HERE / 'test.c', WORKER / 'worker.c', out / 'root-probe.c',
                 BASE / 'text-arena.c', HERE.parent / 'aot-fault-details/render.c', obj,
                 '-o', binary])
            print(('deferred: ' if defer else 'eager: ') + run([binary]), end='')
        assert sha(aot) == report['aotSha256']
        for name, digest in {**qualification['inputs'], **report['inputs']}.items():
            assert sha(Path(name)) == digest, name
        report['passed'] = True
    finally:
        report['artifacts'] = {p.name: sha(p) for p in out.iterdir() if p.is_file()}
        (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

if __name__ == '__main__':
    main()
