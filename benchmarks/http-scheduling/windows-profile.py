#!/usr/bin/env python3
"""Windows AOT collector, socket-poll and scheduler-pause attribution."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'benchmarks/http-server'))
import benchmark
from importlib import import_module
replace_option = import_module('windows-root-observation').replace_option


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--workload', choices=['plain', 'json'], required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build = json.loads(a.build.read_text())
    assert build['passed']
    cmd = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    inputs = {str(Path(v).resolve()): benchmark.sha(v) for v in cmd if Path(v).is_file()}
    prefix = ('#include <stdint.h>\nextern uint64_t neoclr_wait_profile_now(void);\n'
              'extern void neoclr_wait_profile_poll(uint64_t, int32_t);\n'
              'extern void neoclr_wait_profile_pause(void);\nextern void neoclr_wait_profile_start(void);\n')
    for name in ['benchmarks/native-web/http-host.c', 'docs/experiments/runtime-async/native-runtime.c',
                 'docs/experiments/aot-console/socket-listener.c']:
        source = ROOT / name
        if str(source) not in cmd:
            if name.endswith('native-runtime.c'): continue
            raise ValueError('Missing source: ' + name)
        text = source.read_text()
        if name.endswith('socket-listener.c'):
            signature = 'static int32_t socket_poll(neoclr_aot_context *context, uint64_t *callback) {'
            assert text.count(signature) == 1
            text = text.replace(signature, '''static int32_t socket_poll_inner(neoclr_aot_context *, uint64_t *);
static int32_t socket_poll(neoclr_aot_context *context, uint64_t *callback) {
    uint64_t start = neoclr_wait_profile_now();
    int32_t result = socket_poll_inner(context, callback);
    neoclr_wait_profile_poll(start, result);
    return result;
}
static int32_t socket_poll_inner(neoclr_aot_context *context, uint64_t *callback) {''')
        else:
            assert 'neoclr_os_pause()' in text
            text = text.replace('neoclr_os_pause()', 'neoclr_wait_profile_pause()')
        if name.endswith('http-host.c'):
            main = 'int NEOCLR_PROCESS_MAIN(int argc, neoclr_process_char **argv) {'
            assert text.count(main) == 1
            text = text.replace(main, main + '\n    neoclr_wait_profile_start();')
            text = text.replace('../../tools/native/windows-gc-host.h', (ROOT/'tools/native/windows-gc-host.h').as_posix())
        copy = out / source.name
        copy.write_text(prefix + text)
        cmd = [str(copy) if v == str(source) else v for v in cmd]
    app = out / 'app.exe'
    cmd = replace_option(replace_option(cmd, '/Fe:', app), '/Fo', str(out) + '/')
    counters = Path(__file__).with_name('windows-counters.c').resolve()
    position = cmd.index('/link')
    cmd[position:position] = ['/DNEOCLR_GC_PROFILE', str(counters)]
    for folder in ['aot-console', 'runtime-async', 'aot-fault-details']:
        for path in (ROOT/'docs/experiments'/folder).glob('*.h'): inputs[str(path)] = benchmark.sha(path)
    for path in [Path(__file__).resolve(), counters, a.build.resolve(), *out.glob('*.c')]:
        inputs[str(path)] = benchmark.sha(path)
    report = dict(passed=False, inputs=inputs, command=cmd, workload=a.workload, runs=[],
        scope='Instrumented AOT whole-process wall intervals and process CPU. Poll time may overlap GC; do not sum categories. Includes startup, warmup, client inter-request gaps and shutdown. Not uninstrumented throughput or isolated OS wait latency.')
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        report['compile'] = dict(exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        if result.returncode: raise RuntimeError(result.stdout + result.stderr)
        for n in range(2):
            target = out / f'counters-{n}.json'
            old = os.environ.get('NEOCLR_WAIT_PROFILE_OUTPUT')
            os.environ['NEOCLR_WAIT_PROFILE_OUTPUT'] = str(target)
            try: validation = benchmark.server_sample([str(app)], a.workload)
            finally:
                if old is None: os.environ.pop('NEOCLR_WAIT_PROFILE_OUTPUT', None)
                else: os.environ['NEOCLR_WAIT_PROFILE_OUTPUT'] = old
            data = json.loads(target.read_text())
            assert data['polls'] == data['ready'] + data['empty'] + data['errors']
            assert data['ready'] > 0 and data['errors'] == data['gcFailures'] == 0
            assert data['allocations'] > 0 and data['allocations'] == data['reclaimed']
            report['runs'].append(dict(validation=validation, counters=data))
        for path, digest in inputs.items(): assert benchmark.sha(path) == digest, path
        report['passed'] = True
    finally: (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')

if __name__ == '__main__': main()
