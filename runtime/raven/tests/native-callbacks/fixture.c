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

static int add_one(int value) { return value + 1; }
static int notification_value;
static void notify(int value) { notification_value = value; }

int (*native_function_get(void))(int) { return add_one; }
void (*native_notification_get(void))(int) { return notify; }
int native_notification_value(void) { return notification_value; }
