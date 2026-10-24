"""Qualify immutable shadow capture against existing native collector fixtures."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--sanitizers', nargs='+', default=['none', 'undefined,bounds'])
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
report = dict(passed=False, commands=[])
def run(command):
    command = list(map(str, command))
    result = subprocess.run(command, capture_output=True, text=True, timeout=180)
    report['commands'].append(dict(command=command, exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
    return result.stdout
try:
    source = out / 'native-gc.c'
    run(['python3', Path(__file__).with_name('generate.py'), '--output', source])
    recipe = json.loads((root / 'benchmarks/execution/call-layout/results.json').read_text())['builds'][1]['command']
    cc = recipe[:recipe.index('-std=c11')]
    # Drain each external-heap fixture's final job before process shutdown.
    tail = out / 'drain.c'
    tail.write_text('#include <stdint.h>\n#include <stdio.h>\nextern void experiment_gc_shadow(uint64_t *);\n__attribute__((destructor)) static void drain(void) { uint64_t s[5]; experiment_gc_shadow(s); fprintf(stderr,"SHADOW %llu %llu\\n",(unsigned long long)s[0],(unsigned long long)s[4]); }\n')
    cases = ['lifetime', 'native-gc-test', 'native-gc-scan-test', 'native-gc-index-test', 'native-gc-dense-test', 'native-gc-seed-test', 'native-gc-intern-test', 'native-gc-slot-map-test']
    for sanitizer in args.sanitizers:
        selected = cases if sanitizer != 'thread' else ['lifetime', 'native-gc-test']
        for case in selected:
            test = Path(__file__).with_name('lifetime.c') if case == 'lifetime' else root / 'docs/experiments/aot-console' / (case + '.c')
            exe = out / (case + '-' + sanitizer.replace(',', '-'))
            command = cc + ['-O2', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread', '-DNEOCLR_NATIVE_GC', '-DNEOCLR_SHADOW_TEST', '-I', root / 'runtime/native/gc', test, root / 'runtime/native/gc/root-probe.c', source, tail, '-o', exe]
            if sanitizer != 'none':
                command += ['-fsanitize=' + sanitizer, '-fno-sanitize-recover=all']
            run(command)
            run([exe])
    report['inputs'] = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in [source, *Path(__file__).parent.glob('*.[ch]'), Path(__file__), Path(__file__).with_name('generate.py'), * (root / 'runtime/native/gc').glob('*.[ch]')]}
    report['passed'] = True
finally:
    (out / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
