#!/usr/bin/env python3
"""Prepare current-bundle CPU and JSON cases for the external benchmark runner."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from xml.sax.saxutils import quoteattr

ROOT=Path(__file__).resolve().parents[2]
def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    root,out=a.bundle.resolve(),a.output.resolve()
    m=json.loads((root/'manifest.json').read_text())
    for name,digest in m['files'].items():
        if sha(root/name)!=digest:
            raise ValueError('Bundle drift: '+name)
    out.mkdir(parents=True,exist_ok=False)
    project=out/'project';project.mkdir()
    sources=[ROOT/'benchmarks/execution/Worker.rvn',ROOT/'benchmarks/framework/Worker.rvn']
    for index,source in enumerate(sources):
        (project/f'Source{index}.rvn').write_bytes(source.read_bytes())
    (project/'App.rvnproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><Import Project='+quoteattr(str(root/'lib/NeoCLR.ClassLibrary.props'))+'/><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><AssemblyName>App</AssemblyName></PropertyGroup></Project>')
    native=out/'native'
    with (out/'build.log').open('w') as log:
        subprocess.run(['python3',str(ROOT/'scripts/build-native-project.py'),'--profile','console','--project',str(project/'App.rvnproj'),
          '--bundle',str(root),'--aot',str(root/m['jit']['compiler']),'--output',str(native),
          '--byte-array-limit',str(m['jit']['byteArrayLimit']),'--command-timeout','600'],stdout=log,stderr=subprocess.STDOUT,check=True)
    context=['--execution','interpreter','--system',str(root/m['runtimeSeed']),'--object-root',str(root/m['objectRoot']),'--instructions','100000000']
    for module in m['modules']:
        context+=['--module',str(root/module)]
    runtime=str(root/m['runtime']);assembly=str(native/'app.dll')
    inputs={str(root/name):digest for name,digest in m['files'].items()}
    inputs.update(json.loads((native/'build.json').read_text())['inputs'])
    for path in [Path(__file__).resolve(),*sources,root/'manifest.json',native/'app.dll',native/'app']:
        inputs[str(path)]=sha(path)
    report={'schemaVersion':1,'commands':{'interpreter':[runtime,'run',assembly,*context], 'jit':[runtime,'run',assembly], 'aot':[str(native/'app')]},
      'cases':['cpu.sum','json.serialize','json.deserialize'],'inputs':inputs}
    (out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
    print(out/'manifest.json')

if __name__=='__main__':
    main()
