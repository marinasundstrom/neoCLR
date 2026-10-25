"""Check legacy/custom response matching and wrong-body rejection in the Go client."""
import argparse,hashlib,json,socketserver,subprocess,threading
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--client',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();assert not a.output.exists()
r={'passed':False,'cases':[],'clientSha256':hashlib.sha256(a.client.read_bytes()).hexdigest()}
class Server(socketserver.ThreadingTCPServer):
 allow_reuse_address=True;daemon_threads=True;request_queue_size=128
class Handler(socketserver.BaseRequestHandler):
 def handle(self):
  self.request.settimeout(2);request=b''
  while b'\r\n\r\n' not in request:request+=self.request.recv(4096)
  body=self.server.body
  self.request.sendall(b'HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: '+str(len(body)).encode()+b'\r\n\r\n'+body)
try:
 for body,args,success in [(b'xxxxxxxxxx',[],True),(b'Hello, World!',['Hello, World!'],True),(b'wrong',['Hello, World!'],False)]:
  with Server(('127.0.0.1',0),Handler) as server:
   server.body=body;t=threading.Thread(target=server.serve_forever);t.start()
   try:
    cmd=[str(a.client.resolve()),str(server.server_address[1]),'.05',*args];x=subprocess.run(cmd,capture_output=True,text=True,timeout=15)
    r['cases'].append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr));assert (x.returncode==0)==success
    if success:assert json.loads(x.stdout)['completed']>0
    else:assert 'invalid response' in x.stderr
   finally:server.shutdown();t.join()
 r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
