import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--sparse-index', action='store_true')
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
    cc = recipe[:recipe.index('-std=c11')]
    if args.sparse_index: cc += ['-DNEOCLR_SHADOW_SPARSE_INDEX']
    for sanitizer in [False, True]:
        for case in ['lifetime', 'native-gc-test', 'native-gc-scan-test', 'native-gc-dense-test', 'native-gc-seed-test']:
            fixture = base/'lifetime.c'
            if case != 'lifetime':
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
            command = cc+['-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC','-DNEOCLR_SHADOW_TEST','-I',root/'runtime/native/gc',fixture,root/'runtime/native/gc/root-probe.c',source,'-o',exe]
            if sanitizer: command+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
            run(command); run([exe])
    report['inputs']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source, *base.glob('*.[ch]'), base/'generate.py', base/'validate.py', * (base.parent/'concurrent-shadow').glob('*.[ch]')]}
    report['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
