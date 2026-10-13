#!/usr/bin/env python3
"""Count native socket polls without changing production code or guest objects."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('concurrency', ROOT/'benchmarks/http-concurrency/validate.py')
validation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validation)

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build', type=Path, required=True)
    p.add_argument('--workload', choices=['concurrent', 'plain', 'json'], default='concurrent')
    p.add_argument('--gc-phases', action='store_true')
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build = json.loads(a.build.read_text())
    command = next(c['command'][:] for c in build['commands'] if c.get('phase') == 'host-compile-link')
    command[command.index('-o')+1] = str(out/'app')
    source = ROOT/'docs/experiments/aot-console/socket-listener.c'
    inputs = {str(Path(v).resolve()): sha(v) for v in command if Path(v).is_file()}
    for path in [a.build, Path(__file__), Path(validation.__file__), Path(__file__).with_name('counters.c'), ROOT/'benchmarks/http-server/profile-gc.c']:
        inputs[str(path.resolve())] = sha(path)
    for directory in ['aot-console', 'aot-fault-details', 'runtime-async']:
        for path in (ROOT/'docs/experiments'/directory).glob('*.h'):
            inputs[str(path)] = sha(path)
    original = source.read_text()
    signature = 'static int32_t socket_poll(neoclr_aot_context *context, uint64_t *callback) {'
    assert original.count(signature) == 1
    wrapper = '''extern uint64_t neoclr_schedule_profile_now(void);
extern void neoclr_schedule_profile_poll(uint64_t, int32_t);
static int32_t socket_poll_inner(neoclr_aot_context *, uint64_t *);
static int32_t socket_poll(neoclr_aot_context *context, uint64_t *callback) {
    uint64_t start = neoclr_schedule_profile_now();
    int32_t result = socket_poll_inner(context, callback);
    neoclr_schedule_profile_poll(start, result);
    return result;
}
static int32_t socket_poll_inner(neoclr_aot_context *context, uint64_t *callback) {'''
    diagnostic = out/'socket-profile.c'
    diagnostic.write_text(original.replace(signature, wrapper))
    assert str(source) in command
    command = [str(diagnostic) if v == str(source) else v for v in command]
    command += ['-DNEOCLR_GC_PROFILE', str(ROOT/'benchmarks/http-server/profile-gc.c'), str(Path(__file__).with_name('counters.c').resolve())]
    inputs[str(diagnostic)] = sha(diagnostic)
    host = ROOT/'benchmarks/native-web/http-host.c'
    assert str(host) in command
    host_copy = out/'host-profile.c'
    host_source = host.read_text()
    assert 'neoclr_scheduler_wait_v1(context)' in host_source
    host_copy.write_text('#include \"socket-listener.h\"\nstatic void neoclr_schedule_profile_pause(neoclr_aot_context *);\n' +
        host_source.replace('neoclr_scheduler_wait_v1(context)', 'neoclr_schedule_profile_pause(context)') +
        '\nextern uint64_t neoclr_schedule_profile_now(void);\n'
        'extern void neoclr_schedule_profile_paused(uint64_t);\n'
        'static void neoclr_schedule_profile_pause(neoclr_aot_context *context) {\n'
        '    uint64_t start = neoclr_schedule_profile_now();\n'
        '    neoclr_scheduler_wait_v1(context);\n'
        '    neoclr_schedule_profile_paused(start);\n}\n')
    command = [str(host_copy) if v == str(host) else v for v in command]
    inputs[str(host_copy)] = sha(host_copy)
    carrier = ROOT/'docs/experiments/runtime-async/native-runtime.c'
    runtime_async = str(carrier) in command
    if runtime_async:
        carrier_source = carrier.read_text()
        pause_hook = 'if (!ready) neoclr_os_pause();'
        fiber_hook = '    ++statistics.fibers;'
        assert carrier_source.count(pause_hook) == carrier_source.count(fiber_hook) == 1
        carrier_copy = out/'carrier-profile.c'
        carrier_copy.write_text(carrier_source.replace('#define LIMIT 128',
            'extern uint64_t neoclr_schedule_profile_now(void);\n'
            'extern void neoclr_schedule_profile_paused(uint64_t);\n'
            'extern void neoclr_schedule_profile_fiber(int);\n#define LIMIT 128')
            .replace(pause_hook, 'if (!ready) { uint64_t start = neoclr_schedule_profile_now(); '
                'neoclr_os_pause(); neoclr_schedule_profile_paused(start); }')
            .replace(fiber_hook, fiber_hook+'\n    neoclr_schedule_profile_fiber(drain);'))
        command = [str(carrier_copy) if v == str(carrier) else v for v in command]
        inputs[str(carrier_copy)] = sha(carrier_copy)
    if a.gc_phases:
        import gc_phases
        command, extra = gc_phases.instrument(ROOT, out, command)
        for path in extra: inputs[str(path)] = sha(path)
    benchmark = None
    if a.workload != 'concurrent':
        import sys
        sys.path.insert(0, str(ROOT/'benchmarks/http-server'))
        import benchmark
        inputs[str(Path(benchmark.__file__).resolve())] = sha(benchmark.__file__)
    report = dict(passed=False, inputs=inputs, command=command, workload=a.workload, runtimeAsyncCarrierLinked=runtime_async, gcPhaseProfiling=a.gc_phases, runs=[],
                  scope='Instrumented AOT polling diagnosis; identical guest objects. Carrier presence alone does not establish guest lowering; pair with verified backend binding evidence. Not throughput, wakeup-to-resume latency, JIT or interpreter evidence. GC and clock hooks perturb timings.')
    try:
        subprocess.run(command, check=True, capture_output=True, text=True)
        # Repeat both scenarios; the intentional delay applies only after all fast
        # peers finish, when the last peer cannot make progress without more bytes.
        for index, hold in enumerate([0, .1, .1, 0] if a.workload == 'concurrent' else [0, 0]):
            counters, gc = out/f'polls-{index}.json', out/f'gc-{index}.json'
            run_command = ['/usr/bin/env', 'NEOCLR_SCHEDULING_PROFILE_OUTPUT='+str(counters),
                'NEOCLR_HTTP_PROFILE_OUTPUT='+str(gc), str(out/'app')]
            phase_file = out/f'phases-{index}.json'
            if a.gc_phases: run_command.insert(2, 'NEOCLR_GC_PHASE_OUTPUT='+str(phase_file))
            result = (validation.run_case(run_command, slow_hold_seconds=hold)
                      if a.workload == 'concurrent' else benchmark.server_sample(run_command, a.workload))
            poll = json.loads(counters.read_text())
            collector = json.loads(gc.read_text().splitlines()[0])
            assert poll['polls'] == poll['ready'] + poll['empty'] + poll['errors']
            assert poll['ready'] > 0 and poll['errors'] == 0
            assert collector['allocations'] == collector['reclaimed'] and collector['failures'] == 0
            report['runs'].append(dict(holdSecondsPerPhase=hold, validation=result, scheduler=poll, gc=collector))
            if a.gc_phases:
                phases = json.loads(phase_file.read_text())
                assert phases['collections'] == collector['attempts'] and phases['failures'] == 0
                assert 0 < phases['boundaryCalls'] <= phases['collections']
                report['runs'][-1]['gcPhases'] = phases
            print(index, hold, poll, flush=True)
        for path, digest in inputs.items():
            assert sha(path) == digest, path
        report['passed'] = True
    finally:
        (out/'profile.json').write_text(json.dumps(report, indent=2)+'\n')

if __name__ == '__main__':
    main()
