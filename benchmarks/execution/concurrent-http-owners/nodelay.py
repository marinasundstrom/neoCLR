"""Relink an existing owner workload with experimental POSIX TCP_NODELAY; no default change."""
import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
source=root/'docs/experiments/aot-console';r={'passed':False,'sample':'plain','commands':[],'inputs':{},'control':str(base)}
try:
 original=json.loads((base/'results.json').read_text());assert original['passed'] and original['sample']=='plain'
 header=(source/'socket-os.h').read_text();needle='    int flags = fcntl(fd, F_GETFL), fdflags = fcntl(fd, F_GETFD);';assert header.count(needle)==1
 header=header.replace('#include <arpa/inet.h>','#include <arpa/inet.h>\n#include <netinet/tcp.h>')
 header=header.replace(needle,'    int one = 1;\n    if (setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &one, sizeof(one))) return -1;\n'+needle)
 header=header.replace('#define NEOCLR_SOCKET_OS_H','#define NEOCLR_SOCKET_OS_H\n#define NEOCLR_NODELAY_ACTIVE 1',1)
 (out/'socket-os.h').write_text(header)
 (out/'socket-listener.c').write_text('#include \"socket-os.h\"\n#ifndef NEOCLR_NODELAY_ACTIVE\n#error Experimental socket header inactive\n#endif\n'+(source/'socket-listener.c').read_text())
 for c in original['commands']:
  if '-o' not in c['command']:continue
  cmd=[str(out/'socket-listener.c') if x==str(source/'socket-listener.c') else x for x in c['command']]
  cmd[cmd.index('-o')+1]=str(out/Path(cmd[cmd.index('-o')+1]).name)
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
  for item in cmd:
   path=Path(item)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),base/'results.json',source/'socket-os.h',out/'socket-os.h']:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
