from pathlib import Path
import gzip,json,subprocess,hashlib,os
root=Path.cwd();w=root/'target/selection-box-scan-20261028';bundle=w/'release/neoclr-native-poc';out=w/'hosts';out.mkdir(exist_ok=True)
log=[]
def run(c,env=None):
 r=subprocess.run(list(map(str,c)),env=env,capture_output=True,text=True);log.append(dict(command=r.args,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));(w/'bootstrap.json').write_text(json.dumps(log,indent=2));r.check_returncode();return r
run(['python3','runtime/native/gc/concurrent/generate.py','--output',out/'native-gc.c'])
run(['python3','runtime/native/gc/concurrent/native-providers.py','--output',out/'text-arena.c'])
s=gzip.decompress(Path('benchmarks/execution/concurrent-workloads/stack-jit-owned-host.c.gz').read_bytes()).decode();s=s[:s.index('static void report_candidate(void) {')];s=s.replace('static void report_candidate(void);','').replace('    report_candidate();','')
boundary='    int32_t status = run_entry(entry, with_args, argc, argv);'
assert s.count(boundary)==1
s=s.replace(boundary,'''    struct rusage before, after;
    struct timespec start, end;
    getrusage(RUSAGE_SELF, &before);
    clock_gettime(CLOCK_MONOTONIC, &start);
'''+boundary+'''
    clock_gettime(CLOCK_MONOTONIC, &end);
    getrusage(RUSAGE_SELF, &after);
    report_profile(&before, &after, &start, &end);
''')
# Reuse the standard measurement implementation exactly, without executing its CLI.
text=Path('benchmarks/execution/concurrent-workloads/measure.py').read_text();start=text.index('static double cpu(struct rusage *r)');end=text.index("\n''')",start);profile=text[start:end].replace('\\\\n','\\n')
host=out/'owned-host.c';host.write_text('#include <sys/resource.h>\n#include <time.h>\nstatic void report_profile(struct rusage *, struct rusage *, struct timespec *, struct timespec *);\n'+s+profile+'\n')
build=json.load(gzip.open('benchmarks/gc-suite/matrix/http-build.json.gz','rt'));links=[c['command'] for c in build['commands'] if '-dynamiclib' in c['command']]
for mode,link in zip(['sync','live'],links):
 replacements={}
 for v in link:
  if v.endswith('/owned-host.c'):replacements[v]=str(host)
  if mode=='live' and v.endswith('/native-gc.c'):replacements[v]=str(out/'native-gc.c')
  if mode=='live' and v.endswith('/text-arena.c'):replacements[v]=str(out/'text-arena.c')
 link=[replacements.get(v,v).replace('benchmarks/execution/concurrent-mutation-gate','runtime/native/gc/concurrent') for v in link]
 link[link.index('-isysroot')+1]='/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk';link[link.index('-o')+1]=str(out/(mode+'.dylib'))
 run(link)
# Build a fresh guest against the chosen release libraries in a copied project.
import shutil
project=w/'guest';project.mkdir(exist_ok=True)
for n in ['Benchmark.rvn','Benchmark.rvnproj']:shutil.copy2(root/'benchmarks/http-server/pipelining'/n,project/n)
p=project/'Benchmark.rvnproj';p.write_text(p.read_text().replace('../../../runtime/raven',str(root/'runtime/raven')))
# The project references ../Model.rvn.
shutil.copy2(root/'benchmarks/http-server/Model.rvn',w/'Model.rvn')
run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',project/'Benchmark.rvnproj'],dict(os.environ,NeoClrBundleRoot=str(bundle)))
assembly=project/'bin/neoclr/BenchmarkTests.dll'
import runpy
reflection=runpy.run_path(str(bundle/'tools/run-native-sample.py'))['reflection_roots'](bundle/'bin/neoclr',assembly,['Person'],run)
base=json.load(gzip.open('benchmarks/execution/concurrent-http-owners/forward-evidence/http-owner-forwarded-sync-selection.json.gz','rt'))
for mode in ['sync','live']:
 selection=[v.replace(str(root/'target/uri-efficiency-bundle/bundle'),str(bundle)) for v in base];selection[2]=str(assembly);selection[4]=str(out/(mode+'.dylib'));selection.insert(1,'--jit-managed');selection+=list(map(str,reflection));(out/(mode+'-selection.json')).write_text(json.dumps(selection,indent=2)+'\n')
print('Prepared hosts and guest')
