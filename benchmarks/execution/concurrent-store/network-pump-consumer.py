"""Use real loopback scheduler delivery with the generated live async pump."""
import argparse
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();base=Path(__file__).resolve().parent
s=(base/'async-pump-consumer.c').read_text()
s=s.replace('(&c.text,56,NEOCLR_GC_OBJECT','(&c.text,64,NEOCLR_GC_OBJECT')
start=s.index('void neoclr_scheduler_wait_v1(')
end=s.index('extern void experiment_gc_store_counts',start)
s=s[:start]+s[end:]
s=s.replace('neoclr_task_scope scope;', 'neoclr_socket_scope sockets;\n        CHECK(!neoclr_socket_scope_enter_v1(&sockets,&c));\n        neoclr_task_scope scope;')
s=s.replace('CHECK(!neoclr_task_scope_leave_v1(&scope));','CHECK(!neoclr_socket_scope_leave_v1(&sockets));\n            CHECK(!neoclr_task_scope_leave_v1(&scope));')
needle='        CHECK(!neoclr_runtime_async_shutdown_v1(&c));\n        /* A started callback'
assert s.count(needle)==1
s=s.replace(needle,'''        /* A real accept delivery must dispatch the generated callback through
         * the generated pump, while only host ownership retains its descriptor. */
        CHECK(!create(owner+4,&c));
        uint64_t network_operation=owner[4];
        int32_t completed=0;
        CHECK(!complete(network_operation,&completed,&c) && completed);
        uint64_t *network_callback;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&network_callback));
        network_callback[0]=UINT32_MAX;network_callback[1]=1;network_callback[2]=(uintptr_t)owner;
        const struct { uint64_t length; char bytes[16]; } address={9,"127.0.0.1"};
        uint64_t result[2];
        CHECK(!neoclr_socket_listen_v1((const void *)&address,0,4,&c,result) && result[0]==5);
        uint64_t listener=result[1];
        CHECK(!neoclr_socket_local_port_v1(listener,&c,result) && result[0]==1);
        struct sockaddr_in endpoint={0};
        endpoint.sin_family=AF_INET;endpoint.sin_port=htons((uint16_t)result[1]);
        endpoint.sin_addr.s_addr=htonl(INADDR_LOOPBACK);
        CHECK(!neoclr_socket_accept_v1(listener,network_callback,&c,result) && result[0]==5);
        uint64_t accepting=result[1];
        CHECK(!experiment_gc_store(&c,owner+7,accepting,8));
        CHECK(!neoclr_gc_collect_v1(&c,NULL));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!experiment_gc_store(&c,owner+2,99,8));
        neoclr_socket_descriptor peer=socket(AF_INET,SOCK_STREAM,0);
        CHECK(peer!=NEOCLR_INVALID_SOCKET && !connect(peer,(const void *)&endpoint,sizeof(endpoint)));
        CHECK(!pump(&pump_result,&c) && !pump_result && owner[2]==42 && owner[3]);
        CHECK(!sockets.scheduler_delivery && !neoclr_root_probe_head_v1());
        CHECK(!owner[7]);
        CHECK(neoclr_socket_connect_result_v1(accepting,&c,result)==3);
        neoclr_os_close(peer);
        CHECK(!neoclr_socket_close_v1(listener,&c,result) && result[0]==0);
        CHECK(!release(network_operation,&completed,&c) && completed);
'''+needle)
s=s.replace('PASS 100 generated async pump cycles:', 'PASS 100 real-scheduler pump cycles (75 loopback accepts):')
a.output.write_text(s)
