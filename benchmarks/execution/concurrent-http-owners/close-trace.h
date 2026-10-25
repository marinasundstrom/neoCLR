/* Diagnostic-only POSIX adapter. One file per owner; no managed roots or hooks. */
#include <stdio.h>
#include <pthread.h>
static _Thread_local int trace_fd = -1;
static _Thread_local unsigned trace_events;
static void trace_emit(char event, int fd, int local, int peer, long long value, int error) {
    int saved = errno;
    if (trace_fd < 0) {
        const char *prefix = getenv("NEOCLR_CLOSE_TRACE");
        if (!prefix) abort();
        char path[2048];
        int length = snprintf(path,sizeof(path),"%s-%d-%lu.log",prefix,getpid(),(unsigned long)pthread_self());
        if (length < 0 || (size_t)length >= sizeof(path)) abort();
        trace_fd = open(path,O_WRONLY|O_CREAT|O_APPEND|O_CLOEXEC,0600);
        if (trace_fd < 0) abort();
    }
    if (++trace_events > 2000000) abort(); /* Never silently truncate evidence. */
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC,&now)) abort();
    char line[200];
    int length=snprintf(line,sizeof(line),"%llu %c %d %d %d %lld %d\n",
        (unsigned long long)now.tv_sec*1000000000+now.tv_nsec,event,fd,local,peer,value,error);
    if (length<0 || (size_t)length>=sizeof(line) || write(trace_fd,line,(size_t)length)!=length) abort();
    errno=saved;
}
static void trace_ports(int fd,int *local,int *peer) {
    int saved=errno;struct sockaddr_in a={0},b={0};socklen_t n=sizeof(a);
    *local=getsockname(fd,(struct sockaddr *)&a,&n)?0:ntohs(a.sin_port);n=sizeof(b);
    *peer=getpeername(fd,(struct sockaddr *)&b,&n)?0:ntohs(b.sin_port);errno=saved;
}
static int trace_close(int fd) {
    int local,peer;trace_ports(fd,&local,&peer);trace_emit('B',fd,local,peer,0,0);
    int result=close(fd),error=result?errno:0;
    trace_emit('C',fd,local,peer,result,error);errno=error;return result;
}
