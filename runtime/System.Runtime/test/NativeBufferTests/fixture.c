/* Exact synchronous C ABI fixture; never retains the borrowed pointer. */
void native_buffer_increment(unsigned char *buffer, int count) {
    for (int i = 0; i < count; ++i) buffer[i] += 1;
}
