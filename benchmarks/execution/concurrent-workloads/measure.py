"""Measure admitted batched samples with matched owned sync/live AOT hosts.

Compile every host before timing. Native invocation CPU excludes compilation and
process startup; includes console output, cleanup and collector worker CPU.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rounds', type=int, default=5)
p.add_argument('--guarded-control', action='store_true')
p.add_argument('--disable-exact-store', action='store_true')
p.add_argument('--store-ranges', type=int, choices=range(1,17), default=4)
a = p.parse_args()
assert a.rounds > 0
root = Path(__file__).resolve().parents[3]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads((a.build / 'results.json').read_text())
recipe_file = root / 'target/native-host-ownership/results.json'
recipe = next(c['command'] for c in json.loads(recipe_file.read_text())['commands']
              if str(root / 'target/native-host-ownership/owned-aot') in c['command'])
modes = ['sync', 'guarded-sync', 'live'] if a.guarded_control else ['sync', 'live']
report = {'passed': False, 'scope': __doc__, 'iterations': build['iterations'],
          'commands': [], 'trials': [], 'inputs': {}, 'excluded': []}
def run(command):
    result = subprocess.run(list(map(str, command)), cwd=root, capture_output=True,
                            text=True, timeout=180)
    row = {'command': list(map(str, command)), 'exitCode': result.returncode,
           'stdout': result.stdout, 'stderr': result.stderr}
    report['commands'].append(row)
    result.check_returncode()
    return result
try:
    collector = out / 'native-gc.c'
    provider = out / 'text-arena.c'
    run(['python3', root / 'benchmarks/execution/concurrent-live/native-scopes.py', '--output', collector])
    run(['python3', root / 'benchmarks/execution/concurrent-store/native-providers.py', '--output', provider])
    guarded = out / 'guarded-sync-gc.c'
    if a.guarded_control:
        code = collector.read_text()
        start = 'candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;'
        idle = 'if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'
        assert code.count(start) == 1 and code.count(idle) == 1
        guarded.write_text(code.replace(start, 'candidate_autostart = 0; /* Synchronous control. */').replace(idle, 'if (0 && !status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'))
    host = out / 'owned-host.c'
    source = (root / 'target/native-host-ownership/owned-host.c').read_text()
    boundary = '    int32_t status = run_entry(entry, with_args, argc, argv);'
    assert source.count(boundary) == 1
    source = source.replace(boundary, '''    struct rusage before, after;
    struct timespec start, end;
    getrusage(RUSAGE_SELF, &before);
    clock_gettime(CLOCK_MONOTONIC, &start);
''' + boundary + '''
    clock_gettime(CLOCK_MONOTONIC, &end);
    getrusage(RUSAGE_SELF, &after);
    report_profile(&before, &after, &start, &end);
''')
    host.write_text('''#include <sys/resource.h>
#include <time.h>
static void report_profile(struct rusage *, struct rusage *, struct timespec *, struct timespec *);
''' + source + '''
static double cpu(struct rusage *r) {
 return r->ru_utime.tv_sec + r->ru_stime.tv_sec +
        (r->ru_utime.tv_usec + r->ru_stime.tv_usec) / 1000000.0;
}
static void report_profile(struct rusage *a, struct rusage *b, struct timespec *s, struct timespec *e) {
 neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
 neoclr_gc_profile profile = neoclr_gc_profile_v1();
 neoclr_gc_lookup_profile lookup = neoclr_gc_lookup_profile_v1();
 fprintf(stderr, "LOOKUPS %llu %llu %llu %llu\\n",
  (unsigned long long)lookup.base_calls, (unsigned long long)lookup.base_cache_hits,
  (unsigned long long)lookup.base_walks, (unsigned long long)lookup.base_spans);
 fprintf(stderr, "PROFILE %.9f %.9f %llu %llu %llu %llu %llu\\n",
  cpu(b)-cpu(a), (e->tv_sec-s->tv_sec)+(e->tv_nsec-s->tv_nsec)/1000000000.0,
  (unsigned long long)stats.allocations, (unsigned long long)profile.allocated_bytes,
  (unsigned long long)stats.collections, (unsigned long long)profile.peak_live_bytes,
  (unsigned long long)profile.peak_arena_used);
#ifdef WORKLOAD_LIVE
 extern void experiment_gc_candidate_counts(uint64_t *);
 extern void experiment_gc_store_counts(uint64_t *);
 extern void experiment_gc_store_lookup_counts(uint64_t *);
 uint64_t counts[6]; experiment_gc_candidate_counts(counts);
 fprintf(stderr, "CANDIDATE");
 for(unsigned i=0;i<6;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
 experiment_gc_store_counts(counts);
 fprintf(stderr,"STORES");
 for(unsigned i=0;i<5;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\\n");
 experiment_gc_store_lookup_counts(counts);
 fprintf(stderr,"STORE_LOOKUPS %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1]);
#endif
}
''')
    admitted = []
    for case in build['cases']:
        if len(case['native']) != 2 or any(n['exitCode'] for n in case['native']):
            report['excluded'].append(case['sample'])
            continue
        admitted.append(case)
        for mode in modes:
            command = []
            for item in recipe:
                if item.endswith('/owned-host.c'): item = str(host)
                elif item == str(root / 'target/root-map/candidate.o'):
                    item = str(a.build.resolve() / case['sample'] / (('sync' if mode == 'sync' else 'live') + '.o'))
                elif mode != 'sync' and item.endswith('/native-gc.c'): item = str(guarded if mode == 'guarded-sync' else collector)
                elif mode != 'sync' and item.endswith('/text-arena.c'): item = str(provider)
                command.append(item)
            command[command.index('-o') + 1] = str(out / (case['sample'] + '-' + mode))
            command += ['-DNEOCLR_GC_PROFILE', f'-DNEOCLR_LIVE_STORE_RANGES={a.store_ranges}']
            if mode != 'sync':
                gate = root / 'benchmarks/execution/concurrent-mutation-gate'
                command += ['-DWORKLOAD_LIVE', '-DNEOCLR_SHADOW_SPARSE_INDEX',
                            '-I', str(root / 'runtime/native/memory'), '-I', str(gate), str(gate / 'gate.c')]
            if a.disable_exact_store: command += ['-DNEOCLR_LIVE_NO_EXACT_STORE']
            run(command)
            for item in command:
                path = Path(item)
                if path.is_file(): report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for repetition in range(a.rounds):
        for case in admitted:
            for mode in (modes if repetition % 2 == 0 else list(reversed(modes))):
                result = subprocess.run([str(out / (case['sample'] + '-' + mode))],
                                        capture_output=True, text=True, timeout=120)
                row = {'sample': case['sample'], 'mode': mode, 'round': repetition,
                       'exitCode': result.returncode, 'stderr': result.stderr,
                       'stdout': result.stdout, 'outputMatches': result.stdout == case['expectedPerIteration'] * build['iterations']}
                report['trials'].append(row)
                lines = result.stderr.splitlines()
                for label in ['LOOKUPS', 'STORES', 'STORE_LOOKUPS']:
                    row[label] = next((list(map(int, x.split()[1:])) for x in lines if x.startswith(label + ' ')), None)
                values = next((x.split()[1:] for x in lines if x.startswith('PROFILE ')), None)
                if values:
                    row.update(dict(zip(['cpuSeconds', 'wallSeconds'], map(float, values[:2]))))
                    row.update(dict(zip(['allocations', 'allocatedBytes', 'collections', 'peakLiveBytes', 'peakArenaUsed'], map(int, values[2:]))))
                if mode != 'sync':
                    counts = next((list(map(int, x.split()[1:])) for x in lines if x.startswith('CANDIDATE ')), [])
                    row['candidateCounts'] = counts
                row['passed'] = result.returncode == 0 and row['outputMatches'] and values is not None
                if mode != 'sync': row['passed'] &= len(counts) == 6 and counts[0] == counts[1] + counts[2] and counts[5] == 0
                if mode == 'guarded-sync': row['passed'] &= counts == [0] * 6
    report['allocationEquality'] = all(
        len({(t.get('allocations'), t.get('allocatedBytes')) for t in report['trials'] if t['sample'] == c['sample']}) == 1
        for c in admitted)
    report['passed'] = report['allocationEquality'] and bool(report['trials']) and all(t['passed'] for t in report['trials'])
    for path in [Path(__file__), a.build / 'results.json', recipe_file]:
        report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(0 if report['passed'] else 1)
