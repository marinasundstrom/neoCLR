/* Private live-GC host query. Not a public hosting API or shared-heap promise.
 * Querying must not allocate, collect, mutate roots or start a worker. Unknown
 * versions return zero. Hosts own and must qualify the implementations behind
 * these declarations; this handshake is compatibility checking, not isolation. */
#define NEOCLR_LIVE_HOST_VERSION UINT32_C(1)
#define NEOCLR_LIVE_NONCOLLECTING_STORE UINT64_C(1)
#define NEOCLR_LIVE_OLD_NEW_LOGGING UINT64_C(2)
#define NEOCLR_LIVE_OWNED_NONMOVING UINT64_C(4)
#define NEOCLR_LIVE_BOUNDARY_FINISH UINT64_C(8)
#define NEOCLR_LIVE_REQUIRED (NEOCLR_LIVE_NONCOLLECTING_STORE | NEOCLR_LIVE_OLD_NEW_LOGGING | NEOCLR_LIVE_OWNED_NONMOVING)
#ifndef NEOCLR_LIVE_NATIVE_SCOPES
#define NEOCLR_LIVE_NATIVE_SCOPES UINT64_C(0)
#endif
#ifdef NEOCLR_LIVE_STORE_HANDLE
#define NEOCLR_LIVE_BORROWED_STORE UINT64_C(32)
#else
#define NEOCLR_LIVE_BORROWED_STORE UINT64_C(0)
#endif
uint64_t experiment_gc_live_capabilities_v1(uint32_t version) {
    return version==NEOCLR_LIVE_HOST_VERSION ? (NEOCLR_LIVE_REQUIRED | NEOCLR_LIVE_BOUNDARY_FINISH | NEOCLR_LIVE_BORROWED_STORE | NEOCLR_LIVE_NATIVE_SCOPES) : 0;
}
