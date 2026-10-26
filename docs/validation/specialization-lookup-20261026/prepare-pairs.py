from pathlib import Path
import copy,hashlib,json,subprocess
root=Path('/Users/robert/Projects/neoclr');work=root/'target/specialization-lookup-20261026'
base=json.loads((work/'baseline/build.json').read_text())
for name,digest in base['inputs'].items():
 assert hashlib.sha256(Path(name).read_bytes()).hexdigest()==digest,name
out=work/'paired';out.mkdir(exist_ok=False)
b=copy.deepcopy(base);b['recipes']={};b['scope']='Alternating retained-baseline/indexed-specialization-source pairs; production-default codegen and unchanged guest/native hosts. Source snapshots distinguish variants.'
b['sourceRevision']='7396dfe74 plus archived specialization-source patch'
b['workingHeadAtPreparation']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
runner=work/'indexed-runner';runner.write_bytes((root/'target/release/deps/neoclr_codegen-52405fc785913ebe').read_bytes());runner.chmod(0o755)
b['inputs'][str(runner)]=hashlib.sha256(runner.read_bytes()).hexdigest()
for mode,recipe in base['recipes'].items():
 b['recipes']['baseline-'+mode]=recipe
 indexed=copy.deepcopy(recipe);indexed['command'][0]=str(runner)
 b['recipes']['indexed-'+mode]=indexed
for name in ['src/metadata.rs','src/runtime_lookup.rs','runtime/codegen/src/selection.rs','runtime/codegen/src/jit_lazy.rs','runtime/codegen/src/specialization.rs']:
 path=root/name;dest=out/path.name;dest.write_bytes(path.read_bytes());b['inputs'][str(dest)]=hashlib.sha256(dest.read_bytes()).hexdigest()
path=Path(__file__);b['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(b,indent=2)+'\n')
print(' '.join(b['recipes']))
