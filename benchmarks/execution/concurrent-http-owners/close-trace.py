"""Relink checked hosts with bounded send/close diagnostics; not performance evidence."""
import argparse,json,subprocess,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--build',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];base=a.build.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
source=root/'docs/experiments/aot-console';r={'passed':False,'sample':'plain','commands':[],'inputs':{},'control':str(base),'diagnostic':'send/close trace'}
try:
 original=json.loads((base/'results.json').read_text());assert original['passed'] and original['sample']=='plain'
 header=(source/'socket-os.h').read_text();needle='static inline int neoclr_os_close(neoclr_socket_descriptor fd) { return close(fd); }';assert header.count(needle)==1
 header=header.replace(needle,'#include <stdlib.h>\n#include "close-trace.h"\nstatic inline int neoclr_os_close(neoclr_socket_descriptor fd) { return trace_close(fd); }')
 needle='    return sending ? send(fd, buffer, count, flags) : recv(fd, buffer, count, 0);';assert header.count(needle)==1
 header=header.replace(needle,'''    int64_t result = sending ? send(fd, buffer, count, flags) : recv(fd, buffer, count, 0);
    int saved=errno;
    if (sending && result>0) { int local,peer;trace_ports(fd,&local,&peer);trace_emit('S',fd,local,peer,result,0); }
    errno=saved;return result;''')
 header=header.replace('#define NEOCLR_SOCKET_OS_H','#define NEOCLR_SOCKET_OS_H\n#define NEOCLR_CLOSE_TRACE_ACTIVE 1',1)
 (out/'socket-os.h').write_text(header)
 (out/'socket-listener.c').write_text('#include \"socket-os.h\"\n#ifndef NEOCLR_CLOSE_TRACE_ACTIVE\n#error Experimental socket header inactive\n#endif\n'+(source/'socket-listener.c').read_text())
 (out/'close-trace.h').write_text(Path(__file__).with_suffix('.h').read_text())
 for c in original['commands']:
  if '-o' not in c['command']:continue
  cmd=[str(out/'socket-listener.c') if x==str(source/'socket-listener.c') else x for x in c['command']]
  cmd[cmd.index('-o')+1]=str(out/Path(cmd[cmd.index('-o')+1]).name)
  x=subprocess.run(cmd,capture_output=True,text=True,timeout=60);r['commands'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));x.check_returncode()
  for item in cmd:
   path=Path(item)
   if path.is_file():r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 for path in [Path(__file__),base/'results.json',source/'socket-os.h',out/'socket-os.h',out/'close-trace.h']:r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
 r['passed']=True
finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
