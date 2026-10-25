"""Prepare matching private HTTP benchmark artifacts before any timing begins."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import runpy

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[3]
base=root/'benchmarks/http-server/pipelining'
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
bundle=root/'target/uri-efficiency-bundle/bundle'
runner=root/'target/release/deps/neoclr_codegen-12f33f26d2ff4df0'
assembly=base/'bin/neoclr/BenchmarkTests.dll'
r=dict(passed=False,scope='Build only; execute smoke and measured clients separately',commands=[],recipes={},inputs={})

def run(command,env=None):
    command=list(map(str,command))
    x=subprocess.run(command,cwd=root,env=env,capture_output=True,text=True,timeout=300)
    r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
    x.check_returncode()

try:
    run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',base/'Benchmark.rvnproj'],dict(os.environ,NeoClrBundleRoot=str(bundle)))
    reflection=list(map(str,runpy.run_path(str(bundle/'tools/run-native-sample.py'))['reflection_roots'](
        bundle/'bin/neoclr',assembly,['Person'],run)))
    original=json.loads((root/'target/gc-patterns-standard-build/compute/live-command.json').read_text())
    context=original[original.index('--system'):original.index('--compile-system')]
    r['recipes']['interpreter']=dict(execution='interpreter',collector='mark-sweep',env={},
        command=list(map(str,[bundle/'bin/neoclr','run',assembly,'--execution','interpreter','--gc-stats',
                              '--instructions','4000000000',*context])))
    for mode,report,suffix in [('sync','target/gc-suite-standard-v1/standalone/results.json','compute-sync'),
                               ('live','target/gc-index-only-paired/results.json','compute-live-budget-1024-16384-index-only-hybrid-inline')]:
        previous=json.loads((root/report).read_text())
        assert previous['passed']
        link=next(c['command'].copy() for c in previous['commands'] if '-o' in c['command'] and c['command'][c['command'].index('-o')+1].endswith(suffix))
        selection=original.copy()+reflection;selection[2]=str(assembly);selection[4]=str(out/(mode+'.o'))
        path=out/(mode+'-selection.json');path.write_text(json.dumps(selection,indent=2)+'\n')
        settings=dict(NEOCLR_TEST_OPT_LEVEL='speed',NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE=mode)
        env=dict(os.environ)
        for flag in ['NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS','NEOCLR_LIVE_FORWARD_STACK_ARGUMENTS',
                     'NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS','NEOCLR_LIVE_SSA_BYREF_ARGUMENTS']:
            env.pop(flag,None)
            if mode=='live':settings[flag]='1'
        env.update(settings)
        launch=list(map(str,[runner,'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']))
        run(launch,env)
        link=[str(out/(mode+'.o')) if v.endswith('/compute/'+mode+'.o') else v for v in link]
        assert str(out/(mode+'.o')) in link
        link[link.index('-o')+1]=str(out/mode)
        if mode=='sync':
            assert not any(v.startswith(('-DNEOCLR_GC_ALLOCATION_BUDGET=', '-DNEOCLR_GC_PUBLICATION_BUDGET=')) for v in link)
            link += ['-DNEOCLR_GC_ALLOCATION_BUDGET=256', '-DNEOCLR_GC_PUBLICATION_BUDGET=4096']
        if mode=='live':
            assert '-DNEOCLR_GC_ALLOCATION_BUDGET=1024' in link
            assert '-DNEOCLR_GC_PUBLICATION_BUDGET=16384' in link
            assert '-DNEOCLR_LIVE_NO_STORE_CACHE' in link
        run(link)
        r['recipes'][mode+'-aot']=dict(execution='aot',collector=mode,command=[str(out/mode)],env={})
        shared=[v for v in link if not v.endswith('/main.c') and v!=str(out/(mode+'.o'))]+['-dynamiclib']
        host=out/(mode+'.dylib');shared[shared.index('-o')+1]=str(host);run(shared)
        jit=[selection[0],'--jit-managed',*selection[1:4],str(host),*selection[5:]]
        jit_path=out/(mode+'-jit-selection.json');jit_path.write_text(json.dumps(jit,indent=2)+'\n')
        settings['NEOCLR_LIVE_SELECTION_COMMAND']=str(jit_path)
        r['recipes'][mode+'-jit']=dict(execution='jit',collector=mode,command=launch,env=settings)
    paths=[Path(__file__),base/'Benchmark.rvn',base/'Benchmark.rvnproj',base.parent/'Model.rvn',assembly,assembly.with_suffix('.reflection.json'),runner,
           bundle/'tools/run-native-sample.py',
           root/'target/gc-patterns-standard-build/compute/live-command.json']
    paths+=list((root/'runtime/raven/src/System/Web').rglob('*.rvn'))
    paths+=[out/'sync',out/'live']
    paths+=list(out.glob('*.o'))+list(out.glob('*.dylib'))+list(out.glob('*selection.json'))
    for path in paths:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed']=True
finally:
    (out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
