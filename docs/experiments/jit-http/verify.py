#!/usr/bin/env python3
"""Qualify HTTP server, client/DNS and timeout cleanup through JIT/AOT/interpreter."""
import argparse
import hashlib
import json
from pathlib import Path
import queue
import platform
import socket
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--bundle', type=Path, required=True)
    prerequisite = p.add_mutually_exclusive_group(required=True)
    prerequisite.add_argument('--console-qualification', type=Path)
    prerequisite.add_argument('--windows-build', type=Path)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--cli', action='store_true', help='Invoke JIT through neoclr run --execution jit')
    p.add_argument('--mode', action='append', choices=['interpreter', 'aot', 'jit'], help='Select relevant modes; defaults to three-mode parity')
    a = p.parse_args()
    out, bundle = a.output.resolve(), a.bundle.resolve()
    out.mkdir(parents=True, exist_ok=False)
    windows = a.windows_build is not None
    if windows and platform.system() != 'Windows':
        raise ValueError('Windows qualification requires Windows execution')
    prior = json.loads((a.windows_build or a.console_qualification).read_text())
    if not prior['passed']:
        raise ValueError('Console qualification required')
    jit = ROOT/'tools/aot-poc/target/release/neoclr-aot-poc'
    runtime = ROOT/('target/release/neoclr.exe' if windows else 'target/release/neoclr')
    compiler = bundle/'sdk/tools/rvnc/rvnc.dll'
    lib = bundle/'lib'
    refs = [lib/(n+'.dll') for n in ['System.Runtime','System.Data','System.Networking','System.Web']]
    seed = lib/'System.runtime.neox'
    context = ['--system',seed,'--object-root',refs[0]]
    for ref in refs:
        context += ['--module',ref]
    source_compile = ['dotnet',compiler,'neoclr','--core-reference',lib/'Core.dll','--runtime-seed',seed,
                      '--bootstrap-intrinsics','--bootstrap-ownership',lib/'ownership.json',
                      '--object-library','System.Runtime','--async-library','System.Runtime']
    for ref in refs:
        source_compile += ['--reference',ref]
    if windows:
        import windows as windows_host
        native_command, host_command = windows_host.recipe(prior, ROOT, out)
        jit = Path(native_command[0])
        native_base = native_command[1:]
        old_input, old_host = native_base[1], native_base[3]
        host = out/'host.dll'
    else:
        native_base = prior['jitCommand'][1:]
        native_base.remove('--jit-managed')
        old_input, old_host = native_base[1], native_base[3]
        host = out/'host.dylib'
        host_command = [str(host) if part == old_host else part for part in prior['hostCommand']]
        for name in ['task-queue.c','socket-listener.c']:
            source = str(ROOT/'docs/experiments/aot-console'/name)
            if source not in host_command:
                host_command.append(source)
    report = dict(passed=False, jitViaRuntimeCli=a.cli, sourceBaseRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                  inputs={str(f):sha(f) for f in [jit,runtime,compiler,seed,*refs,lib/'Core.dll',lib/'ownership.json',HERE/'verify.py',ROOT/'docs/experiments/jit-managed/host.c']},
                  builds=[], artifacts={}, cases=[])
    commands = {}

    def build(cmd):
        cmd = list(map(str,cmd))
        r = subprocess.run(cmd,capture_output=True,text=True,timeout=180)
        record = dict(command=cmd,exitCode=r.returncode,stderr=r.stderr)
        if len(r.stdout)>4000:
            record.update(stdoutPrefix=r.stdout[:2000],stdoutSha256=hashlib.sha256(r.stdout.encode()).hexdigest())
        else:
            record['stdout']=r.stdout
        report['builds'].append(record)
        if r.returncode:
            raise RuntimeError(r.stdout[:1000]+r.stderr)
        return r

    def outcome(mode, code, stdout, stderr):
        stdout,stderr=stdout.decode(),stderr.decode()
        result=dict(processExitCode=code,stdout=stdout,stderr=stderr)
        if mode=='jit' and not a.cli:
            summaries=[s for s in stderr.splitlines() if s.startswith('JIT_RESULT ')]
            if code or len(summaries)!=1:
                raise RuntimeError('JIT did not complete: '+stderr)
            result['jit']=json.loads(summaries[0].removeprefix('JIT_RESULT '))
            result['exitCode']=result['jit']['guestExitCode']
            result['stderr']=''.join(s for s in stderr.splitlines(keepends=True) if not s.startswith('JIT_RESULT '))
        else:
            result['exitCode']=code
        return result

    def command(name,mode,args=()):
        cmd=commands[name][mode].copy()
        if mode=='jit' and not a.cli:
            for arg in args: cmd+=['--jit-entry-argument',str(arg)]
        elif mode=='interpreter' or mode=='jit' and a.cli:
            if args: cmd+=['--',*map(str,args)]
        else: cmd+=list(map(str,args))
        return cmd

    def start_server(mode,name='server'):
        cmd=command(name,mode)
        proc=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,bufsize=0)
        try:
            readiness = queue.Queue(maxsize=1)
            reader = threading.Thread(target=lambda: readiness.put(proc.stdout.readline()), daemon=True)
            reader.start()
            port = readiness.get(timeout=45).strip()
            if not port.isdigit() or not 0<int(port)<65536:
                raise RuntimeError('Server failed before readiness: '+repr(port))
            return proc,int(port)
        except BaseException:
            stop(proc)
            raise

    def stop(proc):
        if proc.poll() is None:
            proc.kill()
        proc.communicate(timeout=10)

    expected_tail='Other work runs while HTTP accept is pending\nServed greeting; server closed\n'
    valid=b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
    response=(b'HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n'+'Café 🌍'.encode())
    try:
        if windows:
            windows_host.build_host(host_command, out, build)
        else:
            build(host_command)
        report['hostSha256']=sha(host)
        server_source=ROOT/'docs/experiments/http-server/Server.rvn'
        fault_source=out/'ServerFault.rvn'
        text=server_source.read_text().replace('func Respond(request: HttpRequest) -> Task<Result<HttpResponse, HttpError>> {','func Respond(request: HttpRequest) -> Task<Result<HttpResponse, HttpError>> {\n    System.Fail("HTTP callback fault")')
        fault_source.write_text(text)
        for name,source,with_args in [('server',server_source,False),('client',HERE/'Client.rvn',True),
                                      ('timeout',ROOT/'docs/experiments/http-timeout/Network.rvn',True),('fault',fault_source,False)]:
            folder=out/name;folder.mkdir()
            assembly,obj,exe=(folder/n for n in ['app.dll','app.obj' if windows else 'app.o','app.exe' if windows else 'app'])
            build([*source_compile,'-o',assembly,source])
            native=[str(assembly) if part==old_input else str(obj) if part==old_host else part for part in native_base]
            for flag in ['--bind-task-queue','--bind-timers','--bind-socket-listener','--bind-socket-accept','--bind-socket-transfer','--bind-socket-client']:
                if flag not in native:native.append(flag)
            build([jit,*native])
            wrapper=folder/'main.c'
            entry='neoclr_entry_args_v1' if with_args else 'neoclr_entry_v4'
            argument_type = 'wchar_t' if windows else 'char'
            parameters = 'int32_t,int32_t*,neoclr_aot_context*' + (',const '+argument_type+'**' if with_args else '')
            wrapper.write_text(
                '#include "native-gc.h"\n#include <wchar.h>\n#include <string.h>\n'
                'extern int32_t '+entry+'('+parameters+');\n'
                'extern int32_t neoclr_jit_run_v1(const void*,int32_t,int32_t,const '+argument_type+'**);\n'
                'int '+('wmain' if windows else 'main')+'(int argc,'+argument_type+'**argv){\n'
                'int32_t (*function)('+parameters+') = '+entry+';\n'
                'const void *code;\n_Static_assert(sizeof(code)==sizeof(function), "entry pointer size");\n'
                'memcpy(&code,&function,sizeof(code));\n'
                'return neoclr_jit_run_v1(code,'+str(int(with_args))+',argc,(const '+argument_type+'**)argv);\n}\n')
            if windows:
                build(windows_host.link_guest(host_command, old_host, obj, wrapper, exe, folder))
            else:
                link=[part for part in host_command if part!='-dynamiclib'];link[link.index(str(host))]=str(exe)
                build([*link,obj,wrapper])
            commands[name]={'jit':list(map(str,[jit,'--jit-managed',*[str(host) if part==str(obj) else part for part in native]])),
                            'aot':[str(exe)],'interpreter':list(map(str,[runtime,'run',assembly,*context,'--instructions','100000000']))}
            report['artifacts'][name]=dict(source=str(source),sourceSha256=sha(source),assemblySha256=sha(assembly),objectSha256=sha(obj),executableSha256=sha(exe),commands=commands[name])
            if a.cli:
                commands[name]['jit'] = list(map(str,[runtime,'run',assembly,'--execution','jit',
                    '--jit-compiler',jit,'--jit-host',host,*context]))
            print('BUILT '+name,flush=True)
        raw_cases=[('greeting',valid,False,None),('fragmented',valid,True,None),
                   ('duplicate-length',b'GET /greeting HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n',False,'Duplicate request Content-Length'),
                   ('handler-error',valid.replace(b'/greeting',b'/other'),False,'Unexpected request')]
        for mode in a.mode or ['interpreter','aot','jit']:
            for label,request,fragmented,error in raw_cases:
                proc,port=start_server(mode)
                received=b''
                try:
                    with socket.create_connection(('127.0.0.1',port),timeout=10) as peer:
                        for at in range(0,len(request),3 if fragmented else len(request)):
                            peer.sendall(request[at:at+(3 if fragmented else len(request))])
                            if fragmented:time.sleep(.002)
                        peer.shutdown(socket.SHUT_WR)
                        try:
                            while True:
                                part=peer.recv(4096)
                                if not part:break
                                received+=part
                                if len(received)>8192:raise RuntimeError('Oversized response')
                        except ConnectionResetError:pass
                    stdout,stderr=proc.communicate(timeout=30)
                    r=outcome(mode,proc.returncode,stdout,stderr)
                    r.update(mode=mode,case=label,responseHex=received.hex())
                    report['cases'].append(r)
                    tail=expected_tail if error is None else 'Other work runs while HTTP accept is pending\nServer error: '+error+'\n'
                    if r['exitCode'] or r['stderr'] or r['stdout']!=tail or received!=(response if error is None else b''):
                        raise RuntimeError(str(r))
                finally:stop(proc)
                print('PASS '+mode+' server '+label,flush=True)
            proc,port=start_server(mode)
            try:
                # localhost exercises native DNS and connect, not just numeric-address parsing.
                r=subprocess.run(command('client',mode,[f'http://localhost:{port}/greeting']),capture_output=True,timeout=60)
                client=outcome(mode,r.returncode,r.stdout,r.stderr)
                stdout,stderr=proc.communicate(timeout=30)
                server=outcome(mode,proc.returncode,stdout,stderr)
                report['cases'].append(dict(mode=mode,case='client-server-dns',client=client,server=server))
                if client['exitCode'] or client['stderr'] or client['stdout']!='Café 🌍\n' or server['exitCode'] or server['stderr'] or server['stdout']!=expected_tail:
                    raise RuntimeError(str(report['cases'][-1]))
            finally:stop(proc)
            print('PASS '+mode+' client/server DNS pair',flush=True)
            for case in ['timeout','success']:
                peer_result={}
                with socket.socket() as listener:
                    listener.bind(('127.0.0.1',0));listener.listen(1);listener.settimeout(30)
                    url=f'http://localhost:{listener.getsockname()[1]}/'
                    def serve():
                        try:
                            with listener.accept()[0] as peer:
                                peer.settimeout(10);request=b''
                                while b'\r\n\r\n' not in request:
                                    data=peer.recv(4096)
                                    if not data:raise RuntimeError('Client closed before headers')
                                    request+=data
                                    if len(request)>8192:raise RuntimeError('Oversized request')
                                if case=='success':
                                    time.sleep(.35);peer.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}')
                                peer_result['closed']=peer.recv(1)==b''
                        except Exception as error:peer_result['error']=str(error)
                    worker=threading.Thread(target=serve,daemon=True);worker.start()
                    result=subprocess.run(command('timeout',mode,[url,case]),capture_output=True,timeout=60)
                    worker.join(timeout=12)
                r=outcome(mode,result.returncode,result.stdout,result.stderr)
                report['cases'].append(dict(mode=mode,case='client-'+case,result=r,peer=peer_result))
                expected='PASS socket cleanup before timeout acknowledgement\n' if case=='timeout' else 'PASS delayed response with disabled client timeout\n'
                if r['exitCode'] or r['stderr'] or r['stdout']!=expected or not peer_result.get('closed') or peer_result.get('error') or worker.is_alive():
                    raise RuntimeError(str(report['cases'][-1]))
                print('PASS '+mode+' client '+case,flush=True)
        if a.mode and 'jit' not in a.mode:
            report['passed']=True
            return
        proc,port=start_server('jit','fault')
        try:
            with socket.create_connection(('127.0.0.1',port),timeout=10) as peer:
                peer.sendall(valid);peer.shutdown(socket.SHUT_WR)
                try:closed=peer.recv(1)==b''
                except ConnectionResetError:closed=True
            stdout,stderr=proc.communicate(timeout=30)
            r=outcome('jit',proc.returncode,stdout,stderr)
            report['cases'].append(dict(mode='jit',case='handler-fault',result=r,peerClosed=closed))
            if r['exitCode']!=1 or 'HTTP callback fault' not in r['stderr'] or not closed:
                raise RuntimeError(str(report['cases'][-1]))
        finally:stop(proc)
        print('PASS JIT HTTP callback fault cleanup',flush=True)
        report['passed']=True
    finally:
        (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':main()
