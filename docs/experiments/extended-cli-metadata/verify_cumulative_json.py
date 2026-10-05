"""Artifact-only execution control for the cumulative source-library audit."""
import argparse, json, subprocess, hashlib
from pathlib import Path
root = Path(__file__).resolve().parents[3]
parser = argparse.ArgumentParser(description="Execute two artifact-only JSON consumers against the cumulative source-built library.")
for name in ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
compiler, lib, manifest, seed, core, runtime, out = (getattr(args, name).resolve() for name in
    ('compiler', 'library', 'ownership', 'seed', 'core', 'runtime', 'output'))
out.mkdir(parents=True, exist_ok=False)
common=['dotnet',str(compiler),'neoclr','--core-reference',str(core),'--runtime-seed',str(seed),'--bootstrap-intrinsics','--bootstrap-ownership',str(manifest),'--reference',str(lib)]
cases=[('NativeMapping',[root/'docs/experiments/extended-cli-metadata/bootstrap/json-object-consumer.rvn'],42,'Model constructed\nName assigned\nModel constructed\nInvalid input begins\nInvalid input ends\nNative JSON object mapping passed\n'),('ExistingMapping',[root/'docs/experiments/json-object-mapping'/name for name in ['Mapping.rvn','Main.rvn']],0,'JSON object mapping checks passed\n')]
commands=[]
for name,sources,status,stdout in cases:
 app=out/(name+'.dll')
 for command,expected,output in [(common+['-o',str(app)]+list(map(str,sources)),0,None),([str(runtime),'verify',str(app),'--system',str(seed),'--module',str(lib)],0,None),([str(runtime),'run',str(app),'--instructions','100000000','--system',str(seed),'--module',str(lib)],status,stdout)]:
  r=subprocess.run(command,cwd=root,capture_output=True,text=True,timeout=180);commands.append(dict(command=command,exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr));(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
  if r.returncode!=expected or output is not None and r.stdout!=output:raise Exception(commands[-1])
inputs=[lib,manifest,seed,core,compiler,runtime,Path(__file__)] + [compiler.parent / name for name in ('Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll')]+[p for _,sources,_,_ in cases for p in sources]+list(out.glob('*.dll'))
(out/'validation.json').write_text(json.dumps(dict(runtimeRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),compilerRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=compiler.parent,text=True).strip(),commands=commands,hashes={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}),indent=2)+'\n')
print('PASS')
