//! Checked resource IDs for future managed adapters. Only the host owns Context.
//! No guest integer is interpreted as a native pointer, and no callbacks are retained.
use super::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

const INVALID_INPUT: i32 = -10;
const RESOURCE_LIMIT: i32 = -17;
const INVALID_HANDLE: i32 = -18;
const INVALID_NAME: i32 = -19;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

enum Resource {
    Server(Arc<ServerConfig>),
    Client(Arc<ClientConfig>),
    Session(Box<Session>),
}
struct Context {
    resources: BTreeMap<u64, Resource>,
    limit: usize,
}
impl Context {
    fn insert(&mut self, resource: Resource) -> Result<u64, i32> {
        if self.resources.len() >= self.limit {
            return Err(RESOURCE_LIMIT);
        }
        // Never reuse an ID, even across contexts. Exhaustion fails closed.
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| RESOURCE_LIMIT)?;
        self.resources.insert(id, resource);
        Ok(id)
    }
    fn session(&mut self, id: u64) -> Result<*mut c_void, i32> {
        match self.resources.get_mut(&id) {
            Some(Resource::Session(session)) => Ok((&mut **session as *mut Session).cast()),
            _ => Err(INVALID_HANDLE),
        }
    }
}
unsafe fn context<'a>(pointer: *mut c_void) -> Result<&'a mut Context, i32> {
    if pointer.is_null() {
        return Err(INVALID_INPUT);
    }
    Ok(unsafe { &mut *pointer.cast::<Context>() })
}
unsafe fn input<'a>(pointer: *const u8, count: u32) -> Result<&'a [u8], i32> {
    unsafe { bytes(pointer, count) }.ok_or(INVALID_INPUT)
}
unsafe fn publish(output: *mut u64, create: impl FnOnce() -> Result<u64, i32>) -> i32 {
    if output.is_null() {
        return INVALID_INPUT;
    }
    unsafe { *output = 0 };
    match create() {
        Ok(id) => {
            unsafe { *output = id };
            0
        }
        Err(error) => error,
    }
}

/// Host-owned, serialized context; limit is 1..65536 live resources.
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_tls_context_new(limit: u32) -> *mut c_void {
    if limit == 0 || limit > 65536 {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(Context {
        resources: BTreeMap::new(),
        limit: limit as usize,
    }))
    .cast()
}
/// Deterministic abortive cleanup. Performs no I/O and does not retain callbacks.
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// Consume the context exactly once; no resources may be accessed afterward.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_free(pointer: *mut c_void) {
    if !pointer.is_null() {
        drop(unsafe { Box::from_raw(pointer.cast::<Context>()) });
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_release(pointer: *mut c_void, id: u64) -> i32 {
    let Ok(context) = (unsafe { context(pointer) }) else {
        return INVALID_INPUT;
    };
    if context.resources.remove(&id).is_some() {
        0
    } else {
        INVALID_HANDLE
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
/// A nonnull output slot must be aligned, writable, and disjoint from all inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_server_config(
    pointer: *mut c_void,
    cert: *const u8,
    cert_count: u32,
    key: *const u8,
    key_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let config = configuration::server(
                input(cert, cert_count)?,
                input(key, key_count)?,
                input(alpn, alpn_count)?,
            )
            .map_err(|error| error as i32)?;
            context.insert(Resource::Server(Arc::new(config)))
        })
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
/// A nonnull output slot must be aligned, writable, and disjoint from all inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_client_config(
    pointer: *mut c_void,
    roots: *const u8,
    roots_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let config =
                configuration::client(input(roots, roots_count)?, input(alpn, alpn_count)?)
                    .map_err(|error| error as i32)?;
            context.insert(Resource::Client(Arc::new(config)))
        })
    }
}
/// Client configs require the verified UTF-8 peer name. Server configs require empty name.
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
/// A nonnull output slot must be aligned, writable, and disjoint from all inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_session(
    pointer: *mut c_void,
    config: u64,
    name: *const u8,
    name_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let name = input(name, name_count)?;
            let connection = match context.resources.get(&config) {
                Some(Resource::Server(config)) => {
                    if !name.is_empty() {
                        return Err(INVALID_NAME);
                    }
                    ServerConnection::new(config.clone()).map(Connection::Server)
                }
                Some(Resource::Client(config)) => {
                    let name = std::str::from_utf8(name).map_err(|_| INVALID_NAME)?;
                    let name = rustls::pki_types::ServerName::try_from(name.to_owned())
                        .map_err(|_| INVALID_NAME)?;
                    ClientConnection::new(config.clone(), name).map(Connection::Client)
                }
                _ => return Err(INVALID_HANDLE),
            }
            .map_err(|_| configuration::ConfigurationError::ProviderFailure as i32)?;
            // Reuse the existing session constructor and its bounds/state defaults.
            let session = Box::new(make_session(connection));
            context.insert(Resource::Session(session))
        })
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_handshake(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_handshake(session, io, callback) },
        Err(error) => error,
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_shutdown(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_shutdown(session, io, callback) },
        Err(error) => error,
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_abort(pointer: *mut c_void, id: u64) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => {
            unsafe { neoclr_tls_abort(session) };
            0
        }
        Err(error) => error,
    }
}
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// Nonempty data must be writable for count bytes and exclusively borrowed,
/// disjoint from context and callback state.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_transfer(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
    data: *mut u8,
    count: u32,
    sending: i32,
) -> i64 {
    if count as usize > LIMIT || (count != 0 && data.is_null()) || !matches!(sending, 0 | 1) {
        return INVALID_INPUT as i64;
    }
    let data = if data.is_null() {
        std::ptr::NonNull::<u8>::dangling().as_ptr()
    } else {
        data
    };
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_transfer(session, io, callback, data, count, sending) },
        Err(error) => error as i64,
    }
}

/// Private transport-independent poll operation. Serialize a session's operations;
/// retry a pending write with the same plaintext until completion or abort.
#[derive(Clone, Copy, Debug)]
pub enum PumpOperation {
    Handshake,
    Read,
    Write,
    Shutdown,
}
/// Ciphertext consumed/produced is meaningful even when status is pending or failed.
/// Caller must send produced bytes and preserve unconsumed input across polls.
#[derive(Debug, PartialEq, Eq)]
pub struct PumpProgress {
    pub status: i64,
    pub consumed: usize,
    pub produced: usize,
}
struct BufferedWire<'a> {
    input: &'a [u8],
    output: &'a mut [u8],
    consumed: usize,
    produced: usize,
    eof: bool,
}
unsafe extern "C" fn buffered_transfer(
    state: *mut c_void,
    data: *mut u8,
    count: u32,
    sending: i32,
) -> i64 {
    // This callback is called synchronously by the provider and never retained.
    let wire = unsafe { &mut *state.cast::<BufferedWire<'_>>() };
    if sending != 0 {
        let count = (count as usize).min(wire.output.len() - wire.produced);
        if count == 0 {
            return -1;
        }
        let source = unsafe { std::slice::from_raw_parts(data, count) };
        wire.output[wire.produced..wire.produced + count].copy_from_slice(source);
        wire.produced += count;
        count as i64
    } else {
        let count = (count as usize).min(wire.input.len() - wire.consumed);
        if count == 0 {
            return if wire.eof { 0 } else { -1 };
        }
        let destination = unsafe { std::slice::from_raw_parts_mut(data, count) };
        destination.copy_from_slice(&wire.input[wire.consumed..wire.consumed + count]);
        wire.consumed += count;
        count as i64
    }
}

fn pump(
    context: &mut Context,
    id: u64,
    operation: PumpOperation,
    input: &[u8],
    eof: bool,
    output: &mut [u8],
    plaintext: &mut [u8],
) -> PumpProgress {
    let failure = |status| PumpProgress {
        status,
        consumed: 0,
        produced: 0,
    };
    if input.len() > LIMIT
        || output.len() > LIMIT
        || plaintext.len() > LIMIT
        || (matches!(
            operation,
            PumpOperation::Handshake | PumpOperation::Shutdown
        ) && !plaintext.is_empty())
    {
        return failure(INVALID_INPUT as i64);
    }
    let session = match context.session(id) {
        Ok(session) => session,
        Err(status) => return failure(status as i64),
    };
    let mut wire = BufferedWire {
        input,
        output,
        eof,
        consumed: 0,
        produced: 0,
    };
    let pointer = (&mut wire as *mut BufferedWire<'_>).cast();
    let status = unsafe {
        match operation {
            PumpOperation::Handshake => {
                neoclr_tls_handshake(session, pointer, buffered_transfer) as i64
            }
            PumpOperation::Shutdown => {
                neoclr_tls_shutdown(session, pointer, buffered_transfer) as i64
            }
            PumpOperation::Read | PumpOperation::Write => neoclr_tls_transfer(
                session,
                pointer,
                buffered_transfer,
                plaintext.as_mut_ptr(),
                plaintext.len() as u32,
                i32::from(matches!(operation, PumpOperation::Write)),
            ),
        }
    };
    PumpProgress {
        status,
        consumed: wire.consumed,
        produced: wire.produced,
    }
}

/// Private C progress layout, separate from platform-sized Rust slice lengths.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativePumpProgress {
    pub consumed: u32,
    pub produced: u32,
}
fn span(pointer: *const u8, count: usize) -> Option<(usize, usize)> {
    let start = pointer as usize;
    if count != 0 && start == 0 {
        return None;
    }
    Some((start, start.checked_add(count)?))
}
fn overlap(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 != a.1 && b.0 != b.1 && a.0 < b.1 && b.0 < a.1
}
/// Private checked memory transport. Buffers and progress are borrowed only here;
/// nonempty ranges must be valid and pairwise disjoint. Host serializes context.
/// Rejected null/overflow/length/overlap/operation inputs leave progress untouched.
/// The host remains responsible for accessible, aligned memory and context lifetime.
///
/// # Safety
/// A nonnull pointer must identify a live context returned by neoclr_tls_context_new.
/// The caller must serialize access and prevent reentrant access to that context.
/// Input must be readable; output and plaintext must be writable for their
/// declared lengths. Progress must be aligned and writable. All ranges must be
/// pairwise disjoint and disjoint from context-owned memory for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_pump_v1(
    pointer: *mut c_void,
    id: u64,
    operation: i32,
    input: *const u8,
    input_count: u32,
    eof: i32,
    output: *mut u8,
    output_count: u32,
    plaintext: *mut u8,
    plaintext_count: u32,
    progress: *mut NativePumpProgress,
) -> i64 {
    if pointer.is_null()
        || progress.is_null()
        || !(0..=1).contains(&eof)
        || [input_count, output_count, plaintext_count]
            .iter()
            .any(|n| *n as usize > LIMIT)
    {
        return INVALID_INPUT as i64;
    }
    let operation = match operation {
        1 => PumpOperation::Handshake,
        2 => PumpOperation::Read,
        3 => PumpOperation::Write,
        4 => PumpOperation::Shutdown,
        _ => return INVALID_INPUT as i64,
    };
    let spans = [
        span(input, input_count as usize),
        span(output, output_count as usize),
        span(plaintext, plaintext_count as usize),
        span(progress.cast(), std::mem::size_of::<NativePumpProgress>()),
    ];
    for (i, a) in spans.iter().enumerate() {
        let Some(a) = *a else {
            return INVALID_INPUT as i64;
        };
        for b in spans.iter().skip(i + 1) {
            let Some(b) = *b else {
                return INVALID_INPUT as i64;
            };
            if overlap(a, b) {
                return INVALID_INPUT as i64;
            }
        }
    }
    // Zero-sized ranges may use null in the C ABI; never form a null Rust slice.
    let input = if input_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_count as usize) }
    };
    let output = if output_count == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(output, output_count as usize) }
    };
    let plaintext = if plaintext_count == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(plaintext, plaintext_count as usize) }
    };
    let context = unsafe { &mut *pointer.cast::<Context>() };
    let result = pump(context, id, operation, input, eof != 0, output, plaintext);
    unsafe {
        *progress = NativePumpProgress {
            consumed: result.consumed as u32,
            produced: result.produced as u32,
        };
    }
    result.status
}

/// Safe Rust owner for the interpreter adapter. Not a managed/public platform API.
/// Mutable access serializes provider calls; dropping this value releases all resources.
pub struct OwnedContext(Box<Context>);
impl std::fmt::Debug for OwnedContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsContext")
            .field("resources", &self.0.resources.len())
            .finish()
    }
}
impl OwnedContext {
    pub fn new(limit: u32) -> Option<Self> {
        (limit > 0 && limit <= 65536).then(|| {
            Self(Box::new(Context {
                resources: BTreeMap::new(),
                limit: limit as usize,
            }))
        })
    }
    pub fn resource_count(&self) -> usize {
        self.0.resources.len()
    }
    fn pointer(&mut self) -> *mut c_void {
        (&mut *self.0 as *mut Context).cast()
    }
    pub fn client_config(&mut self, roots: &[u8], alpn: &[u8]) -> Result<u64, i32> {
        if roots.len() > LIMIT || alpn.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_client_config(
                self.pointer(),
                roots.as_ptr(),
                roots.len() as u32,
                alpn.as_ptr(),
                alpn.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    pub fn server_config(&mut self, cert: &[u8], key: &[u8], alpn: &[u8]) -> Result<u64, i32> {
        if cert.len() > LIMIT || key.len() > LIMIT || alpn.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_server_config(
                self.pointer(),
                cert.as_ptr(),
                cert.len() as u32,
                key.as_ptr(),
                key.len() as u32,
                alpn.as_ptr(),
                alpn.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    pub fn session(&mut self, config: u64, name: &[u8]) -> Result<u64, i32> {
        if name.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_session(
                self.pointer(),
                config,
                name.as_ptr(),
                name.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    /// One bounded, nonblocking TLS poll over borrowed memory, with no socket or
    /// managed callback. All buffers are at most 64 KiB. EOF means no more encrypted
    /// bytes follow input; an empty input without EOF is merely would-block.
    /// Handshake/shutdown require empty plaintext. Read fills plaintext; Write
    /// borrows it unchanged. Drain output to the transport before reusing it.
    pub fn pump(
        &mut self,
        id: u64,
        operation: PumpOperation,
        input: &[u8],
        eof: bool,
        output: &mut [u8],
        plaintext: &mut [u8],
    ) -> PumpProgress {
        pump(&mut self.0, id, operation, input, eof, output, plaintext)
    }
    pub fn release(&mut self, id: u64) -> i32 {
        unsafe { neoclr_tls_context_release(self.pointer(), id) }
    }
    pub fn abort(&mut self, id: u64) -> i32 {
        unsafe { neoclr_tls_context_abort(self.pointer(), id) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn client() -> Arc<ClientConfig> {
        Arc::new(
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_no_client_auth(),
        )
    }
    #[test]
    fn handles_reject_stale_foreign_and_wrong_kinds_and_enforce_quota() {
        unsafe {
            assert!(neoclr_tls_context_new(0).is_null());
            assert!(neoclr_tls_context_new(65537).is_null());
            let a = neoclr_tls_context_new(1);
            let b = neoclr_tls_context_new(1);
            let id = context(a)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            assert_eq!(
                context(a).unwrap().insert(Resource::Client(client())),
                Err(RESOURCE_LIMIT)
            );
            assert_eq!(context(a).unwrap().session(id), Err(INVALID_HANDLE));
            assert_eq!(neoclr_tls_context_release(b, id), INVALID_HANDLE);
            assert_eq!(neoclr_tls_context_release(a, id), 0);
            assert_eq!(neoclr_tls_context_release(a, id), INVALID_HANDLE);
            let next = context(a)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            assert_ne!(id, next);
            assert_eq!(neoclr_tls_context_abort(a, id), INVALID_HANDLE);
            assert_eq!(
                neoclr_tls_context_abort(std::ptr::null_mut(), next),
                INVALID_INPUT
            );
            neoclr_tls_context_free(a);
            neoclr_tls_context_free(b);
            neoclr_tls_context_free(std::ptr::null_mut());
        }
    }
    #[test]
    fn session_retains_config_until_context_teardown() {
        unsafe {
            let pointer = neoclr_tls_context_new(2);
            let config = client();
            let weak = Arc::downgrade(&config);
            let id = context(pointer)
                .unwrap()
                .insert(Resource::Client(config))
                .unwrap();
            let mut session = 99;
            assert_eq!(
                neoclr_tls_context_session(pointer, id, b"localhost".as_ptr(), 9, &mut session),
                0
            );
            assert_eq!(neoclr_tls_context_release(pointer, id), 0);
            assert!(weak.upgrade().is_some());
            let mut other = 99;
            assert_eq!(
                neoclr_tls_context_session(pointer, session, b"localhost".as_ptr(), 9, &mut other),
                INVALID_HANDLE
            );
            assert_eq!(other, 0);
            assert_eq!(neoclr_tls_context_abort(pointer, session), 0);
            neoclr_tls_context_free(pointer);
            assert!(
                weak.upgrade().is_none(),
                "context teardown releases retained configuration"
            );
        }
    }
    #[test]
    fn failed_creations_clear_ids_and_do_not_consume_capacity() {
        unsafe {
            let pointer = neoclr_tls_context_new(2);
            let null = std::ptr::null();
            let mut id = 99;
            assert_eq!(
                neoclr_tls_context_client_config(pointer, null, 1, null, 0, &mut id),
                INVALID_INPUT
            );
            assert_eq!(id, 0);
            assert_eq!(
                neoclr_tls_context_server_config(pointer, null, 0, null, 0, null, 0, &mut id),
                -11
            );
            assert!(context(pointer).unwrap().resources.is_empty());
            let config = context(pointer)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            for name in [&b""[..], &b"bad name"[..], &b"\xff"[..]] {
                id = 99;
                assert_eq!(
                    neoclr_tls_context_session(
                        pointer,
                        config,
                        name.as_ptr(),
                        name.len() as u32,
                        &mut id
                    ),
                    INVALID_NAME
                );
                assert_eq!(id, 0);
            }
            assert_eq!(context(pointer).unwrap().resources.len(), 1);
            neoclr_tls_context_free(pointer);
        }
    }
}
