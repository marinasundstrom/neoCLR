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
p.add_argument('--compare-watermarks',type=int,nargs='+',choices=range(1,100),default=[])
p.add_argument('--finish-watermark',type=int,choices=range(1,100))
p.add_argument('--live-budgets',nargs='+',default=[],metavar='ALLOCATIONS:PUBLICATIONS')
p.add_argument('--compare-scratch',action='store_true')
p.add_argument('--compare-scalar-validation',action='store_true')
p.add_argument('--compare-linear-seed',action='store_true')
p.add_argument('--compare-direct-seed',action='store_true')
p.add_argument('--borrow-stores',action='store_true',help='Use borrowed native stores for current live objects; retain checked previous-object controls.')
p.add_argument('--compare-worker-tls',action='store_true')
p.add_argument('--compare-inline-base',action='store_true')
p.add_argument('--compare-exact-allocation-index',action='store_true',help='Interleave predecessor-index, hybrid and direct exact-index variants in one cohort.')
p.add_argument('--direct-exact-allocation-index',action='store_true')
p.add_argument('--exact-allocation-index',action='store_true')
p.add_argument('--build', type=Path, required=True)
p.add_argument('--previous-build', type=Path, help='Previous generated objects with the same inlined native host')
p.add_argument('--modes', nargs='+', help='Select unique enabled modes')
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rounds', type=int, default=5)
p.add_argument('--guarded-control', action='store_true')
p.add_argument('--repeat-snapshot-lookup', action='store_true')
p.add_argument('--header-proof', action='store_true')
p.add_argument('--inline-validation', action='store_true')
p.add_argument('--split-tls', action='store_true')
p.add_argument('--serialized-store', action='store_true')
p.add_argument('--disable-exact-store', action='store_true')
p.add_argument('--store-ranges', type=int, choices=range(1,17), default=4)
p.add_argument('--allocation-index',action='store_true')
p.add_argument('--split-store-tls', action='store_true')
p.add_argument('--full-store-walk', action='store_true')
p.add_argument('--profile-store-walks', action='store_true')
p.add_argument('--profile-store-sites', action='store_true')
p.add_argument('--profile-owner-pauses', action='store_true')
p.add_argument('--profile-begin', action='store_true')
p.add_argument('--profile-finish', action='store_true')
p.add_argument('--profile-collection', action='store_true')
a = p.parse_args()
live_budgets={}
for pair in a.live_budgets:
    values=pair.split(':')
    if len(values)!=2 or not all(v.isdecimal() and 0<int(v)<=2**63-1 for v in values):
        p.error('--live-budgets requires positive ALLOCATIONS:PUBLICATIONS pairs')
    allocation,publication=map(int,values)
    name=f'live-budget-{allocation}-{publication}-hybrid-inline'
    if name in live_budgets:p.error('duplicate live budget')
    live_budgets[name]=(allocation,publication)
assert not(a.finish_watermark and a.compare_watermarks)
assert len(set(a.compare_watermarks))==len(a.compare_watermarks)
assert not a.compare_watermarks or a.compare_inline_base
assert not a.profile_collection or a.profile_finish
assert not a.profile_begin or a.profile_owner_pauses
assert not a.profile_finish or a.profile_owner_pauses
assert a.rounds > 0
assert not a.previous_build or a.compare_inline_base
assert not a.compare_linear_seed or a.compare_inline_base
assert not a.compare_direct_seed or a.compare_inline_base
assert not (a.compare_scratch or live_budgets) or a.compare_inline_base
assert not a.compare_worker_tls or (a.compare_inline_base and not a.split_tls)
assert not a.compare_inline_base or a.compare_exact_allocation_index
assert not a.compare_exact_allocation_index or (a.allocation_index and not a.exact_allocation_index and not a.direct_exact_allocation_index)
assert not a.direct_exact_allocation_index or a.exact_allocation_index
assert not a.exact_allocation_index or a.allocation_index
root = Path(__file__).resolve().parents[3]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
build = json.loads((a.build / 'results.json').read_text())
recipe_file = root / 'target/native-host-ownership/results.json'
recipe = next(c['command'] for c in json.loads(recipe_file.read_text())['commands']
              if str(root / 'target/native-host-ownership/owned-aot') in c['command'])
modes = ['sync', 'guarded-sync', 'live'] if a.guarded_control else ['sync', 'live']
if a.compare_exact_allocation_index:
    modes += ['live-hybrid', 'live-direct']
    if a.guarded_control: modes += ['guarded-sync-hybrid', 'guarded-sync-direct']
if a.compare_inline_base:
    modes += ['live-hybrid-inline']
    if a.guarded_control: modes += ['guarded-sync-hybrid-inline']
if a.compare_worker_tls:
    modes += ['live-hybrid-worker-inline']
    if a.guarded_control: modes += ['guarded-sync-hybrid-worker-inline']
if a.previous_build:
    previous = json.loads((a.previous_build / 'results.json').read_text())
    assert previous['passed'] and previous['iterations'] == build['iterations']
    assert {c['sample']: c['expectedPerIteration'] for c in previous['cases']} == {c['sample']: c['expectedPerIteration'] for c in build['cases']}
    modes += ['live-previous-hybrid-inline']
    if a.guarded_control: modes += ['guarded-sync-previous-hybrid-inline']
if a.compare_linear_seed:
    modes += ['live-linear-seed-hybrid-inline']
    if a.guarded_control: modes += ['guarded-sync-linear-seed-hybrid-inline']
if a.compare_direct_seed:
    modes += ['live-direct-seed-hybrid-inline']
    if a.guarded_control: modes += ['guarded-sync-direct-seed-hybrid-inline']
if a.compare_scratch:
    modes += ['live-scratch-hybrid-inline']
modes += [f'live-watermark-{w}-hybrid-inline' for w in a.compare_watermarks]
modes += list(live_budgets)
if a.modes:
    assert len(set(a.modes)) == len(a.modes) and set(a.modes) <= set(modes)
    modes = [m for m in modes if m in a.modes]
report = {'compareWorkerTls':a.compare_worker_tls,'compareInlineBase':a.compare_inline_base,'compareExactAllocationIndex':a.compare_exact_allocation_index,'modeOrder':'rotating forward/reverse cycles' if a.compare_exact_allocation_index else 'alternating forward/reverse','directExactAllocationIndex':a.direct_exact_allocation_index,'exactAllocationIndex':a.exact_allocation_index,'allocationIndex':a.allocation_index,'splitStoreTls':a.split_store_tls,'fullStoreWalk':a.full_store_walk,'repeatSnapshotLookup': a.repeat_snapshot_lookup, 'headerProof': a.header_proof, 'inlineValidation': a.inline_validation, 'splitTls': a.split_tls, 'serializedStore': a.serialized_store, 'optimizationLevel': build.get('optimizationLevel','none'), 'passed': False, 'scope': __doc__, 'iterations': build['iterations'],
          'commands': [], 'trials': [], 'inputs': {}, 'excluded': []}
if a.profile_store_walks:report['diagnosticOnly']='Heap-walk instrumentation; timings are not benchmark evidence.'
if a.profile_owner_pauses:report['diagnosticOnly']='Owner-operation wall histograms; timings are not throughput evidence.'
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
    if a.profile_store_sites:
        report['diagnosticOnly'] = 'Call-site instrumentation; timings are not benchmark evidence.'
        run(['python3', root / 'benchmarks/execution/concurrent-workloads/store-sites.py', '--source', collector])
    guarded = out / 'guarded-sync-gc.c'
    if a.guarded_control:
        code = collector.read_text()
        start = 'candidate_autostart = reason == NEOCLR_GC_TRIGGER_PUBLICATION || reason == NEOCLR_GC_TRIGGER_ALLOCATION;'
        idle = 'if (!status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'
        assert code.count(start) == 1 and code.count(idle) == 1
        guarded.write_text(code.replace(start, 'candidate_autostart = 0; /* Synchronous control. */').replace(idle, 'if (0 && !status && !native_scope_depth && reason == NEOCLR_GC_TRIGGER_IDLE && owned_heap.arena == &context->text)'))
    sync_profile = out / 'sync-profile-gc.c'
    if a.profile_owner_pauses:
        profiler = root / 'benchmarks/execution/concurrent-workloads/owner-pauses.py'
        run(['python3', profiler, '--source', root / 'runtime/native/gc/native-gc.c', '--output', sync_profile])
        for source in [collector] + ([guarded] if a.guarded_control else []):
            run(['python3', profiler, '--source', source, '--output', source, '--live'] + (['--finish-details'] if a.profile_finish else []) + (['--collection-details'] if a.profile_collection else []) + (['--begin-details'] if a.profile_begin else []))
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
#ifdef OWNER_PAUSE_PROFILE
 extern void experiment_owner_pauses_report(void);
 experiment_owner_pauses_report();
#endif
#ifdef BEGIN_PHASE_PROFILE
 extern void experiment_begin_phases_report(void);
 experiment_begin_phases_report();
#endif
#ifdef FINISH_PHASE_PROFILE
 extern void experiment_finish_phases_report(void);
 experiment_finish_phases_report();
#endif
#ifdef COLLECTION_PART_PROFILE
 extern void experiment_collection_parts_report(void);
 experiment_collection_parts_report();
#endif
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
#ifdef NEOCLR_STORE_SITES
 extern void experiment_store_sites_report(void);
 experiment_store_sites_report();
#endif
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
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
 extern void experiment_gc_index_counts(uint64_t *);
 experiment_gc_index_counts(counts);
 fprintf(stderr,"INDEX_METADATA %llu %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
#ifdef NEOCLR_LIVE_WALK_PROFILE
 extern void experiment_gc_store_walk_counts(uint64_t *);
 experiment_gc_store_walk_counts(counts);
 fprintf(stderr,"STORE_WALKS %llu %llu %llu\\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
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
                    origin = a.previous_build if '-previous-' in mode else a.build
                    item = str(origin.resolve() / case['sample'] / (('sync' if mode == 'sync' else 'live') + '.o'))
                elif mode == 'sync' and a.profile_owner_pauses and item.endswith('/native-gc.c'): item = str(sync_profile)
                elif mode != 'sync' and item.endswith('/native-gc.c'): item = str(guarded if mode.startswith('guarded-sync') else collector)
                elif mode != 'sync' and item.endswith('/text-arena.c'): item = str(provider)
                command.append(item)
            command[command.index('-o') + 1] = str(out / (case['sample'] + '-' + mode))
            command += ['-DNEOCLR_GC_PROFILE', f'-DNEOCLR_LIVE_STORE_RANGES={a.store_ranges}']
            if mode != 'sync':
                gate = root / 'benchmarks/execution/concurrent-mutation-gate'
                command += ['-DWORKLOAD_LIVE', '-DNEOCLR_SHADOW_SPARSE_INDEX',
                            '-I', str(root / 'runtime/native/memory'), '-I', str(gate), str(gate / 'gate.c')]
            if (a.direct_exact_allocation_index or mode.endswith('-direct')) and mode != 'sync': command += ['-DNEOCLR_LIVE_DIRECT_EXACT_INDEX']
            if (a.exact_allocation_index or ('-hybrid' in mode or mode.endswith('-direct'))) and mode != 'sync': command += ['-DNEOCLR_LIVE_EXACT_INDEX']
            if mode in live_budgets:
                allocation,publication=live_budgets[mode]
                command += [f'-DNEOCLR_GC_ALLOCATION_BUDGET={allocation}',f'-DNEOCLR_GC_PUBLICATION_BUDGET={publication}']
            if mode!='sync' and a.finish_watermark:command += [f'-DNEOCLR_LIVE_FINISH_WATERMARK={a.finish_watermark}']
            if mode.startswith('live-watermark-'):command += [f'-DNEOCLR_LIVE_FINISH_WATERMARK={mode.split("-")[2]}']
            if mode=='live-scratch-hybrid-inline': command += ['-DNEOCLR_LIVE_REUSE_SCRATCH']
            if '-worker-' in mode: command += ['-DNEOCLR_LIVE_GROUP_WORKER_TLS']
            if mode.endswith('-inline'): command += ['-DNEOCLR_LIVE_INLINE_BASE']
            if a.allocation_index and mode != 'sync': command += ['-DNEOCLR_LIVE_ALLOCATION_INDEX']
            if a.split_store_tls: command += ['-DNEOCLR_LIVE_SPLIT_STORE_TLS']
            if a.full_store_walk: command += ['-DNEOCLR_LIVE_FULL_STORE_WALK']
            if a.profile_store_walks and mode != 'sync': command += ['-DNEOCLR_LIVE_WALK_PROFILE']
            if a.profile_store_sites and mode != 'sync': command += ['-DNEOCLR_STORE_SITES']
            if a.repeat_snapshot_lookup: command += ['-DNEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP']
            if a.header_proof: command += ['-DNEOCLR_LIVE_HEADER_PROOF']
            if a.inline_validation: command += ['-DNEOCLR_LIVE_INLINE_VALIDATION']
            if a.split_tls: command += ['-DNEOCLR_LIVE_SPLIT_TLS']
            if a.serialized_store: command += ['-DNEOCLR_LIVE_SERIALIZED_STORE']
            if a.disable_exact_store: command += ['-DNEOCLR_LIVE_NO_EXACT_STORE']
            if a.profile_owner_pauses: command += ['-DOWNER_PAUSE_PROFILE', '-I', str(root / 'runtime/native/gc')]
            if a.profile_finish and mode != 'sync': command += ['-DFINISH_PHASE_PROFILE']
            if a.profile_begin and mode != 'sync': command += ['-DBEGIN_PHASE_PROFILE']
            if a.profile_collection and mode != 'sync': command += ['-DCOLLECTION_PART_PROFILE']
            if '-linear-seed-' in mode: command += ['-DNEOCLR_LIVE_LINEAR_SEED']
            if '-direct-seed-' in mode: command += ['-DNEOCLR_LIVE_DIRECT_SEED']
            if a.borrow_stores and mode != 'sync' and '-previous-' not in mode: command += ['-DNEOCLR_LIVE_STORE_HANDLE']
            if a.compare_scalar_validation and mode != 'sync' and 'previous' not in mode:
                command += ['-DNEOCLR_GC_SCALAR_STORAGE_VALIDATION']
            run(command)
            for item in command:
                path = Path(item)
                if path.is_file(): report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    for repetition in range(a.rounds):
        order = modes if repetition % 2 == 0 else list(reversed(modes))
        if a.compare_exact_allocation_index:
            cycle = modes if (repetition // len(modes)) % 2 == 0 else list(reversed(modes))
            offset = repetition % len(modes)
            order = cycle[offset:] + cycle[:offset]
        for case in admitted:
            for mode in order:
                result = subprocess.run([str(out / (case['sample'] + '-' + mode))],
                                        capture_output=True, text=True, timeout=120)
                row = {'sample': case['sample'], 'mode': mode, 'round': repetition,
                       'exitCode': result.returncode, 'stderr': result.stderr,
                       'stdout': result.stdout, 'outputMatches': result.stdout == case['expectedPerIteration'] * build['iterations']}
                report['trials'].append(row)
                lines = result.stderr.splitlines()
                for label in ['LOOKUPS', 'STORES', 'STORE_LOOKUPS', 'STORE_WALKS', 'INDEX_METADATA']:
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
                if mode.startswith('guarded-sync'): row['passed'] &= counts == [0] * 6
                if a.allocation_index and mode != 'sync':
                    metadata = row.get('INDEX_METADATA')
                    row['passed'] &= metadata is not None and len(metadata) == 3 and metadata[0] > 0 and metadata[1] == 0 and metadata[2] > 0
    report['allocationEquality'] = all(
        len({(t.get('allocations'), t.get('allocatedBytes')) for t in report['trials'] if t['sample'] == c['sample']}) == 1
        for c in admitted)
    report['passed'] = report['allocationEquality'] and bool(report['trials']) and all(t['passed'] for t in report['trials'])
    if a.previous_build:
        path = a.previous_build / 'results.json'
        report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [Path(__file__), a.build / 'results.json', recipe_file]:
        report['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(0 if report['passed'] else 1)
