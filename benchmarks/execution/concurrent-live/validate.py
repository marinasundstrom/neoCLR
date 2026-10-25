import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import signal

parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--sparse-index', action='store_true')
parser.add_argument('--store-ranges', type=int, choices=range(1,17), default=4)
parser.add_argument('--inline-validation', action='store_true')
parser.add_argument('--split-tls', action='store_true')
parser.add_argument('--serialized-store', action='store_true')
parser.add_argument('--batch', action='store_true')
parser.add_argument('--disable-store-cache', action='store_true')
parser.add_argument('--gate-words', type=int, default=4096)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
base = Path(__file__).resolve().parent
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[])
def run(command):
    command = list(map(str, command))
    result = subprocess.run(command, capture_output=True, text=True, timeout=120)
    report['commands'].append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
try:
    source = out/'native-gc.c'
    run(['python3', base/'generate.py', '--output', source])
    recipe = json.loads((root/'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc = recipe[:recipe.index('-std=c11')] + [f'-DGATE_WORDS={args.gate_words}', f'-DNEOCLR_LIVE_STORE_RANGES={args.store_ranges}']
    if args.inline_validation: cc += ['-DNEOCLR_LIVE_INLINE_VALIDATION']
    if args.split_tls: cc += ['-DNEOCLR_LIVE_SPLIT_TLS']
    if args.serialized_store: cc += ['-DNEOCLR_LIVE_SERIALIZED_STORE']
    if args.sparse_index: cc += ['-DNEOCLR_SHADOW_SPARSE_INDEX']
    if args.disable_store_cache: cc += ['-DNEOCLR_LIVE_NO_STORE_CACHE']
    if args.batch: cc += ['-DNEOCLR_LIVE_BATCH','-DNEOCLR_LIVE_GATE_PROFILE']
    for sanitizer in [False, True]:
        for case in ['lifetime', 'store-test', 'exact-store-test', 'record-store-test', 'native-gc-test', 'native-gc-scan-test', 'native-gc-dense-test', 'native-gc-seed-test']:
            fixture = base/(case+'.c')
            if case not in ['lifetime', 'store-test', 'exact-store-test', 'record-store-test']:
                fixture = out/(case+'.c')
                fixture.write_text('''#include "native-gc.h"
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern void experiment_gc_shadow(uint64_t *);
static int32_t candidate_collect_conditional(neoclr_aot_context *c, const neoclr_probe_frame *f, neoclr_gc_conditional_root *r, uint32_t n) {
 int32_t status = experiment_gc_begin(c,f,r,n);
 return status ? status : experiment_gc_finish(c,f,r,n);
}
__attribute__((unused)) static int32_t candidate_collect(neoclr_aot_context *c, const neoclr_probe_frame *f) { return candidate_collect_conditional(c,f,0,0); }
#define neoclr_gc_collect_v1 candidate_collect
#define neoclr_gc_collect_conditional_v1 candidate_collect_conditional
#include "''' + str(root/'docs/experiments/aot-console'/f'{case}.c') + '''"
__attribute__((destructor)) static void drain(void) { uint64_t stats[5]; experiment_gc_shadow(stats); }
''')
            exe = out/f'{case}-{sanitizer}'
            command = cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-I',base.parent/'concurrent-mutation-gate','-I',root/'runtime/native/gc',base.parent/'concurrent-mutation-gate/gate.c',fixture,root/'runtime/native/gc/root-probe.c',source,'-o',exe]
            if sanitizer: command+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
            run(command); run([exe])
    # Deliberately drop deletion candidates. The delayed sole-edge test must fail.
    gate_source = (base.parent/'concurrent-mutation-gate/gate.c').read_text()
    original = 'memcpy(&g->words[g->count++], (unsigned char *)payload + at, 8);'
    assert gate_source.count(original) == 2
    negative_gate = out/'negative-gate.c'
    negative_gate.write_text(gate_source.replace(original,'g->words[g->count++] = 0;',1))
    negative = out/'negative'
    command = cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-I',base.parent/'concurrent-mutation-gate','-I',root/'runtime/native/gc',negative_gate,base/'lifetime.c',root/'runtime/native/gc/root-probe.c',source,'-o',negative]
    run(command)
    result = subprocess.run([str(negative)],capture_output=True,text=True,timeout=30)
    report['negative'] = dict(exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr,expectedSignal='SIGABRT')
    assert result.returncode == -signal.SIGABRT
    if not args.disable_store_cache:
        # A plausible stale interior header must not substitute for an exact-base
        # proof. Deliberately bypass revocation and require the boundary test to fail.
        unsafe_source=out/'unproven-cache.c'
        guarded='live_store_last_payload && known_base(live_store_last_payload)'
        original_source=source.read_text();assert original_source.count(guarded)==1
        unsafe_source.write_text(original_source.replace(guarded,'live_store_last_payload'))
        unsafe_exe=out/'unproven-cache'
        run(cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-I',base.parent/'concurrent-mutation-gate','-I',root/'runtime/native/gc',base.parent/'concurrent-mutation-gate/gate.c',base/'store-test.c',root/'runtime/native/gc/root-probe.c',unsafe_source,'-o',unsafe_exe])
        result=subprocess.run([str(unsafe_exe)],capture_output=True,text=True,timeout=30)
        report['cacheNegative']=dict(exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr,expectedSignal='SIGABRT')
        assert result.returncode == -signal.SIGABRT
    if args.store_ranges > 1 and not args.disable_store_cache:
        # Revocation must protect every older range, not just the newest hint.
        unsafe_source=out/'unproven-recent.c'
        original_source=source.read_text();assert original_source.count('!known_base(recent)')==1
        unsafe_source.write_text(original_source.replace('!known_base(recent)','0'))
        unsafe_exe=out/'unproven-recent'
        run(cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PROFILE','-DNEOCLR_SHADOW_TEST','-I',base.parent/'concurrent-mutation-gate','-I',root/'runtime/native/gc',base.parent/'concurrent-mutation-gate/gate.c',base/'store-test.c',root/'runtime/native/gc/root-probe.c',unsafe_source,'-o',unsafe_exe])
        result=subprocess.run([str(unsafe_exe)],capture_output=True,text=True,timeout=30)
        report['recentNegative']=dict(exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr,expectedSignal='SIGABRT')
        assert result.returncode == -signal.SIGABRT
    report['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source, *base.glob('*.[ch]'), base/'generate.py', base/'validate.py', * (base.parent/'concurrent-shadow').glob('*.[ch]'), *(base.parent/'concurrent-mutation-gate').glob('gate.[ch]')]}
    report['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
