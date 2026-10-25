#include <pthread.h>
static void *work(void *p) {return p;}
int main(void) {pthread_t t; if(pthread_create(&t,0,work,0)) return 1;return pthread_join(t,0);}
