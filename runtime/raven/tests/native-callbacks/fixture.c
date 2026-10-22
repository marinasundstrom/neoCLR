/* Callbacks run synchronously on the calling thread and are never retained. */
int native_callback_apply(int (*callback)(int), int value) {
    return callback(value);
}
int native_callback_twice(int (*callback)(int), int value) {
    return callback(value) + callback(value + 1);
}

void native_callback_notify(void (*callback)(int), int value) {
    callback(value);
}
