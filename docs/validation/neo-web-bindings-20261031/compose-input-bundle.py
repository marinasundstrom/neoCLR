import pathlib,shutil,subprocess,xml.etree.ElementTree as E,os,json,hashlib
root=pathlib.Path('/Users/robert/Projects/neoclr'); out=root/'target/neo-web-bindings';out.mkdir(exist_ok=True)
bundle=out/'input-bundle'
if not bundle.exists():shutil.copytree(root/'target/neo-web-descriptors/validation3/bundle',bundle)
lib=bundle/'lib'; env=dict(os.environ,NeoClrBundleRoot=str(bundle),DOTNET_ROOT='/Users/robert/.dotnet',PATH='/Users/robert/.dotnet:'+os.environ['PATH'])
for name,source in [('System.Web','runtime/System.Web/src/System.Web/System/**/*.rvn'),('Neo.DependencyInjection.Abstractions','extensions/Neo.DependencyInjection/src/Neo.DependencyInjection.Abstractions/Neo/**/*.rvn'),('Neo.DependencyInjection','extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo/**/*.rvn'),('Neo.Web','extensions/Neo.Web/src/Neo.Web/Neo/**/*.rvn')]:
 work=out/name;work.mkdir(exist_ok=True)
 p=E.Element('Project',Sdk='Microsoft.NET.Sdk');E.SubElement(p,'Import',Project=str(lib/'NeoCLR.ClassLibrary.props'));props=E.SubElement(p,'PropertyGroup')
 for k,v in dict(TargetFramework='net10.0',OutputType='Library',AssemblyName=name,EnableDefaultCompileItems='false',RavenNeoClrRuntimeAsyncHelper='').items():E.SubElement(props,k).text=v
 items=E.SubElement(p,'ItemGroup');E.SubElement(items,'Reference',Remove=('Neo.*;System.Web' if name=='System.Web' else ('Neo.*' if name.startswith('Neo.DependencyInjection') else name)));
 if name=='Neo.DependencyInjection':
  ref=E.SubElement(items,'Reference',Include='Neo.DependencyInjection.Abstractions');E.SubElement(ref,'HintPath').text=str(lib/'Neo.DependencyInjection.Abstractions.dll')
 E.SubElement(items,'Compile',Include=str(root/source));E.indent(p);file=work/(name+'.rvnproj');E.ElementTree(p).write(file,encoding='unicode')
 subprocess.run(['dotnet',str(bundle/'sdk/tools/rvnc/rvnc.dll'),'neoclr','--project',str(file)],env=env,check=True)
 for suffix in ('.dll','.xml','.docs'):
  src=work/'bin/neoclr'/(name+suffix);dst=lib/(name+suffix)
  if src.is_dir():shutil.copytree(src,dst,dirs_exist_ok=True)
  else:shutil.copy2(src,dst)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
catalog=json.loads((lib/'bundle.json').read_text())
for name in ['System.Web','Neo.DependencyInjection.Abstractions','Neo.DependencyInjection','Neo.Web']:
 for p in lib.glob(name+'*'):
  for f in ([p] if p.is_file() else p.rglob('*')):
   if f.is_file():catalog['files'][f.relative_to(lib).as_posix()]=sha(f)
(lib/'bundle.json').write_text(json.dumps(catalog,indent=2)+'\n')
manifest=json.loads((bundle/'manifest.json').read_text());manifest['files']={p.relative_to(bundle).as_posix():sha(p) for p in bundle.rglob('*') if p.is_file() and p!=bundle/'manifest.json'};(bundle/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
