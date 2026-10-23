#include <pthread.h>
#include <stdio.h>
static void *run(void *p) { return p; }
int main(void) { pthread_t t; if(pthread_create(&t,0,run,0))return 1; if(pthread_join(t,0))return 2; puts("PASS"); return 0; }
